//! `agntd` — AgntOS agent daemon.
//!
//! Provides a REPL that routes user input to an LLM-powered tool-calling loop
//! (when `/etc/agntos/models.toml` is configured) or falls back to keyword-based
//! command parsing.
//!
//! ## Architecture
//!
//! ```text
//! User input ─► main REPL
//!                  │
//!          ┌───────┴────────┐
//!          ▼                ▼
//!    models.toml?       models.toml?
//!    YES → agent.rs     NO  → keyword handlers
//!    (LLM tool loop)        (propose → confirm → apply)
//!          │
//!    ┌─────┴──────┐
//!    │ llm.rs     │  OpenAI-compatible client, tool definitions, prompt builder
//!    │ agent.rs   │  Turn handler, tool execution, session history
//!    │ session.rs │  SQLite FTS5 store for turn recall
//!    │ util.rs    │  agntctl subprocess, user I/O, /proc fallback
//!    └───────────┘
//! ```
//!
//! ## Key modules
//!
//! - [`agent`] — LLM-powered agent turn (tool-calling loop).
//! - [`llm`] — OpenAI-compatible HTTP client, tool schemas, system prompt assembly.
//! - [`session`] — SQLite FTS5 session store for historical search.
//! - [`util`] — Shared helpers: `agntctl` subprocess, confirmation prompt, `/proc`-based inspection fallback.

mod agent;
mod log;
mod llm;
mod session;
mod turn_guard;
mod util;
mod watchdog;

use agent::LlmSession;
use serde_json::json;
use session::SessionStore;
use std::io::{self, BufRead, BufReader, Write};
use std::os::unix::net::UnixListener;
use std::os::unix::net::UnixStream;
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;

use crate::agent::{execute_tool_call_gui, get_pending_proposal_ids, SharedApprovalGate};
use crate::turn_guard::TurnGuard;
use crate::watchdog::EventSender;
use agnt_common::wire::{AuditRequestAction, ClientMessage, ServerMessage, ToolCallStatus};

// ── Entry point ──────────────────────────────────────────────────────────────

fn main() {
    let args: Vec<String> = std::env::args().collect();

    match args.get(1).map(|s| s.as_str()) {
        Some("--socket") | Some("-s") => {
            let path = args
                .get(2)
                .map(|s| s.as_str())
                .unwrap_or("/run/agntd/agent.sock");
            run_socket_mode(path);
        }
        _ => run_repl(),
    }
}

/// Runs the REPL — interactive mode (original behaviour).
fn run_repl() {
    util::log_startup_paths();
    watchdog::start(util::config_dir_str());

    print_banner();

    let mut llm_state = init_llm_session();
    print_llm_status(&llm_state);

    loop {
        print!("agnt> ");
        io::stdout().flush().unwrap();

        let input = match util::read_line() {
            Some(line) => line.trim().to_string(),
            None => {
                println!();
                break;
            }
        };

        if input.is_empty() {
            continue;
        }

        let lower = input.to_lowercase();

        match lower.as_str() {
            "quit" | "exit" | "bye" => {
                if let Some((runtime, state)) = &llm_state {
                    println!("\nReviewing session for memory-worthy facts...");
                    match agent::end_of_session_review(runtime, &state.client, &state.session_store)
                    {
                        Ok(msg) => println!("  {}", msg),
                        Err(e) => println!("  Memory review skipped: {}", e),
                    }
                }
                println!("bye.");
                break;
            }
            "help" | "?" => {
                show_help(&llm_state);
                continue;
            }
            _ => {}
        }

        if lower.starts_with("history ") {
            if let Some((runtime, state)) = llm_state.as_mut() {
                let _ = agent::agent_turn(&input, runtime, state);
            } else {
                println!("  LLM mode not active. Configure /etc/agntos/models.toml first.");
            }
            continue;
        }

        if let Some((runtime, state)) = llm_state.as_mut() {
            if let Err(e) = agent::agent_turn(&input, runtime, state) {
                println!("  Agent error: {}", e);
            }
            continue;
        }

        handle_keyword_fallback(&lower);
    }
}

/// Runs the socket / daemon mode — listens on a Unix domain socket.
///
/// Detects between:
/// - Legacy one-shot `{"prompt": "..."}` requests (backward compatible)
/// - Typed `{"type": "init", ...}` messages (persistent session protocol)
///
/// For persistent sessions, each connection exchanges NDJSON lines over
/// a long-lived connection.
fn run_socket_mode(socket_path: &str) {
    // Ensure parent directory exists
    if let Some(parent) = Path::new(socket_path).parent() {
        let _ = std::fs::create_dir_all(parent);
    }

    let _ = std::fs::remove_file(socket_path);
    let listener = match UnixListener::bind(socket_path) {
        Ok(l) => {
            // Make the socket world-writable so any user (e.g. the desktop
            // user) can connect to agntd without XDG_RUNTIME_DIR auth issues.
            use std::os::unix::fs::PermissionsExt;
            let _ = std::fs::set_permissions(socket_path, std::fs::Permissions::from_mode(0o777));
            l
        }
        Err(e) => {
            eprintln!("agntd: failed to bind socket at {}: {}", socket_path, e);
            std::process::exit(1);
        }
    };

    util::log_startup_paths();

    let bootstrap = match agent::DaemonBootstrap::from_config_dir(&util::config_dir_str()) {
        Ok(b) => b,
        Err(e) => {
            log::error(&format!("failed to initialise: {}", e));
            eprintln!("agntd: failed to initialise: {}", e);
            let _ = std::fs::remove_file(socket_path);
            std::process::exit(1);
        }
    };

    // Create the event broadcast channel and start watchdog with events
    let event_tx = watchdog::create_event_channel();
    watchdog::start(util::config_dir_str());

    println!(
        "agntd: listening on {} (profile={}, model={})",
        socket_path, bootstrap.client.profile_name, bootstrap.client.profile.model
    );

    for stream in listener.incoming() {
        let stream = match stream {
            Ok(s) => s,
            Err(e) => {
                eprintln!("agntd: connection error: {}", e);
                continue;
            }
        };

        // Read the first message to determine protocol
        let mut reader = BufReader::new(&stream);
        let mut first_line = String::new();
        if reader.read_line(&mut first_line).map_or(true, |n| n == 0) {
            continue;
        }
        let first_line = first_line.trim().to_string();

        if agnt_common::wire::is_legacy_prompt(&first_line) {
            // Legacy one-shot path — backward compatible
            let response = match serde_json::from_str::<serde_json::Value>(&first_line) {
                Ok(req) => match req.get("prompt").and_then(|v| v.as_str()) {
                    Some(prompt) => match agent::process_prompt(prompt, &bootstrap) {
                        Ok(text) => json!({"response": text}),
                        Err(e) => json!({"error": e}),
                    },
                    None => json!({"error": "Missing 'prompt' field"}),
                },
                Err(_) => json!({"error": "Invalid JSON"}),
            };
            let payload = response.to_string();
            let _ = write!(&stream, "{}", payload);
        } else {
            // Persistent session path
            match serde_json::from_str::<ClientMessage>(&first_line) {
                Ok(ClientMessage::Init { .. }) => {
                    if let Err(e) = handle_persistent_session(&bootstrap, stream, &event_tx) {
                        eprintln!("agntd: session error: {}", e);
                    }
                }
                Ok(_) => {
                    let err = json!({"error": "Expected init message first"});
                    let _ = write!(&stream, "{}", err.to_string());
                }
                Err(e) => {
                    let err = json!({"error": format!("Invalid JSON: {}", e)});
                    let _ = write!(&stream, "{}", err.to_string());
                }
            }
        }
    }
}

/// Handles a persistent NDJSON session over a Unix domain socket.
///
/// Protocol:
/// 1. Server sends `session_ready` with profile/model/pending proposals
/// 2. Client sends `chat` messages → server streams tool_calls, tool_results, turn_complete
/// 3. Client sends `approve`/`dismiss` to gate apply/rollback operations
/// 4. Client sends `status`/`audit` for queries
///
/// Chat processing runs on a separate thread to avoid blocking the reader loop.
/// The approval gate is shared between the reader (which resolves it on `approve`/`dismiss`)
/// and the chat thread (which waits on it for apply/rollback).
fn handle_persistent_session(
    bootstrap: &agent::DaemonBootstrap,
    stream: UnixStream,
    _event_tx: &EventSender,
) -> Result<(), String> {
    let read_stream = stream
        .try_clone()
        .map_err(|e| format!("Failed to clone stream: {}", e))?;
    let mut writer = stream;
    let reader = BufReader::new(read_stream);

    let approval_gate: SharedApprovalGate = Arc::new(Mutex::new(None));
    let chatting = Arc::new(AtomicBool::new(false));

    let session_msg = ServerMessage::SessionReady {
        profile: bootstrap.client.profile_name.clone(),
        model: bootstrap.client.profile.model.clone(),
        pending_proposals: get_pending_proposal_ids(&util::config_dir_str()),
    };
    writeln!(writer, "{}", serde_json::to_string(&session_msg).unwrap())
        .map_err(|e| format!("write error: {}", e))?;

    for line in reader.lines() {
        let line = match line {
            Ok(l) => l,
            Err(_) => break,
        };
        if line.trim().is_empty() {
            continue;
        }
        let msg: ClientMessage = match serde_json::from_str(&line) {
            Ok(m) => m,
            Err(e) => {
                let err_msg = ServerMessage::Error {
                    message: format!("Invalid message: {}", e),
                };
                let _ = writeln!(writer, "{}", serde_json::to_string(&err_msg).unwrap());
                continue;
            }
        };

        match msg {
            ClientMessage::Chat { prompt } => {
                if chatting.swap(true, Ordering::SeqCst) {
                    let err_msg = ServerMessage::Error {
                        message: "Already processing a chat. Send cancel or wait.".to_string(),
                    };
                    let _ = writeln!(&writer, "{}", serde_json::to_string(&err_msg).unwrap());
                    continue;
                }

                let mut writer_clone = writer
                    .try_clone()
                    .map_err(|e| format!("write clone failed: {}", e))?;
                let client = bootstrap.client.clone();
                let tools = bootstrap.tools.clone();
                let gate = approval_gate.clone();
                let busy = chatting.clone();
                let config_dir = util::config_dir_str();
                let prompt_clone = prompt.clone();

                thread::spawn(move || {
                    let runtime = match tokio::runtime::Runtime::new() {
                        Ok(r) => r,
                        Err(e) => {
                            let err = ServerMessage::Error {
                                message: format!("Runtime error: {}", e),
                            };
                            let _ = writeln!(
                                &mut writer_clone,
                                "{}",
                                serde_json::to_string(&err).unwrap()
                            );
                            busy.store(false, Ordering::SeqCst);
                            return;
                        }
                    };

                    let inspect_summary = util::capture_inspect("system");
                    let system_prompt =
                        crate::llm::build_system_prompt(&config_dir, &inspect_summary);
                    let mut messages = vec![
                        json!({"role": "system", "content": system_prompt}),
                        json!({"role": "user", "content": prompt_clone}),
                    ];

                    let mut guard = TurnGuard::new();
                    'turn: loop {
                        if let Some(reason) = guard.record_llm_step() {
                            let done = ServerMessage::TurnComplete { content: reason };
                            let _ = writeln!(
                                &mut writer_clone,
                                "{}",
                                serde_json::to_string(&done).unwrap()
                            );
                            break;
                        }

                        let resp = match runtime.block_on(client.complete_streaming_to_writer(
                            &messages,
                            &tools,
                            &mut writer_clone,
                        )) {
                            Ok(r) => r,
                            Err(e) => {
                                let err_msg = ServerMessage::Error {
                                    message: format!("LLM error: {}", e),
                                };
                                let _ = writeln!(
                                    &mut writer_clone,
                                    "{}",
                                    serde_json::to_string(&err_msg).unwrap()
                                );
                                break;
                            }
                        };
                        messages.push(resp.assistant_message.clone());

                        if resp.tool_calls.is_empty() {
                            let done = ServerMessage::TurnComplete {
                                content: resp.content,
                            };
                            let _ = writeln!(
                                &mut writer_clone,
                                "{}",
                                serde_json::to_string(&done).unwrap()
                            );
                            break;
                        }

                        for tc in &resp.tool_calls {
                            let tc_msg = ServerMessage::ToolCall {
                                id: tc.id.clone(),
                                name: tc.name.clone(),
                                args: tc.arguments.clone(),
                                status: ToolCallStatus::Running,
                            };
                            let _ = writeln!(
                                &mut writer_clone,
                                "{}",
                                serde_json::to_string(&tc_msg).unwrap()
                            );

                            if tc.name == "apply" || tc.name == "rollback" {
                                let pid = tc.arguments.get("proposal_id").and_then(|v| v.as_str());
                                let summary = pid.map_or_else(
                                    || "Roll back NixOS generation".to_string(),
                                    |id| format!("Apply proposal {}", id),
                                );
                                let approval_msg = ServerMessage::ApprovalRequest {
                                    proposal_id: pid.unwrap_or("rollback").to_string(),
                                    summary: summary.clone(),
                                    tool_call_id: tc.id.clone(),
                                };
                                let _ = writeln!(
                                    &mut writer_clone,
                                    "{}",
                                    serde_json::to_string(&approval_msg).unwrap()
                                );
                            }

                            let result = execute_tool_call_gui(tc, Some(&prompt), gate.clone());

                            if tc.name == "propose" {
                                if let Ok(ref out) = result {
                                    if let Some(pid) = util::extract_proposal_id(out) {
                                        let summary = util::extract_proposal_summary(out)
                                            .unwrap_or_else(|| "Apply configuration change".to_string());
                                        let approval_msg = ServerMessage::ApprovalRequest {
                                            proposal_id: pid,
                                            summary: summary.clone(),
                                            tool_call_id: tc.id.clone(),
                                        };
                                        let _ = writeln!(
                                            &mut writer_clone,
                                            "{}",
                                            serde_json::to_string(&approval_msg).unwrap()
                                        );
                                    }
                                }
                            }
                            let tool_ok = result.is_ok();

                            match &result {
                                Ok(output) => {
                                    let tr_msg = ServerMessage::ToolResult {
                                        id: tc.id.clone(),
                                        name: tc.name.clone(),
                                        output: output.clone(),
                                        success: true,
                                    };
                                    let _ = writeln!(
                                        &mut writer_clone,
                                        "{}",
                                        serde_json::to_string(&tr_msg).unwrap()
                                    );
                                }
                                Err(e) => {
                                    let tr_msg = ServerMessage::ToolResult {
                                        id: tc.id.clone(),
                                        name: tc.name.clone(),
                                        output: e.clone(),
                                        success: false,
                                    };
                                    let _ = writeln!(
                                        &mut writer_clone,
                                        "{}",
                                        serde_json::to_string(&tr_msg).unwrap()
                                    );
                                }
                            }

                            // Push tool result into conversation history for the next LLM call
                            let tool_content = match &result {
                                Ok(output) => output.clone(),
                                Err(e) => e.clone(),
                            };
                            messages.push(serde_json::json!({
                                "role": "tool",
                                "tool_call_id": tc.id.clone(),
                                "content": tool_content,
                            }));

                            if let Some(reason) =
                                guard.record_tool(&tc.name, &tc.arguments, tool_ok)
                            {
                                let done = ServerMessage::TurnComplete { content: reason };
                                let _ = writeln!(
                                    &mut writer_clone,
                                    "{}",
                                    serde_json::to_string(&done).unwrap()
                                );
                                break 'turn;
                            }
                        }
                    }

                    *gate.lock().unwrap() = None;
                    busy.store(false, Ordering::SeqCst);
                });
            }
            ClientMessage::Approve { proposal_id } => {
                {
                    let mut g = approval_gate.lock().unwrap();
                    if let Some(ref mut state) = *g {
                        if state.proposal_id == proposal_id {
                            state.resolved = true;
                            state.approved = true;
                        }
                    }
                }
                let cfg = util::config_dir_str();
                let pid = proposal_id.clone();
                let mut writer_apply = writer
                    .try_clone()
                    .map_err(|e| format!("write clone failed: {}", e))?;
                thread::spawn(move || {
                    let result = util::run_agntctl(&["apply", "--config-dir", &cfg, &pid]);
                    let (output, success) = match result {
                        Ok((stdout, stderr, ok)) => {
                            let mut text = stdout;
                            if !stderr.is_empty() {
                                if !text.is_empty() {
                                    text.push('\n');
                                }
                                text.push_str(&stderr);
                            }
                            (text, ok)
                        }
                        Err(e) => (e, false),
                    };
                    let tr = ServerMessage::ToolResult {
                        id: pid.clone(),
                        name: "apply".to_string(),
                        output,
                        success,
                    };
                    let _ = writeln!(
                        &mut writer_apply,
                        "{}",
                        serde_json::to_string(&tr).unwrap()
                    );
                });
            }
            ClientMessage::Dismiss {
                proposal_id,
                reason: _,
            } => {
                let mut g = approval_gate.lock().unwrap();
                if let Some(ref mut state) = *g {
                    if state.proposal_id == proposal_id {
                        state.resolved = true;
                        state.approved = false;
                    }
                }
            }
            ClientMessage::Status { target } => {
                let output = util::capture_inspect(&target);
                let status_msg = ServerMessage::StatusResponse {
                    target: target.clone(),
                    data: serde_json::json!({"output": output}),
                };
                let _ = writeln!(writer, "{}", serde_json::to_string(&status_msg).unwrap());
            }
            ClientMessage::Audit {
                action,
                query,
                id,
                limit,
            } => {
                let cfg = util::config_dir_str();
                let cmd = match action {
                    AuditRequestAction::List => util::run_agntctl(&[
                        "audit",
                        "list",
                        "--json",
                        "--limit",
                        &limit.to_string(),
                        "--config-dir",
                        &cfg,
                    ]),
                    AuditRequestAction::Search => {
                        let q = query.unwrap_or_default();
                        util::run_agntctl(&[
                            "audit",
                            "search",
                            "--json",
                            "--query",
                            &q,
                            "--limit",
                            &limit.to_string(),
                            "--config-dir",
                            &cfg,
                        ])
                    }
                    AuditRequestAction::Show => {
                        let id = id.unwrap_or_default();
                        util::run_agntctl(&["audit", "show", "--json", &id, "--config-dir", &cfg])
                    }
                };
                let entries: Vec<serde_json::Value> = match cmd {
                    Ok((stdout, _, _)) => {
                        match serde_json::from_str::<serde_json::Value>(&stdout) {
                            Ok(serde_json::Value::Array(arr)) => arr,
                            Ok(other) => vec![other],
                            Err(_) => vec![],
                        }
                    }
                    Err(_) => vec![],
                };
                let audit_msg = ServerMessage::AuditResponse { entries };
                let _ = writeln!(writer, "{}", serde_json::to_string(&audit_msg).unwrap());
            }
            ClientMessage::Cancel {} => {
                chatting.store(false, Ordering::SeqCst);
                let cancel_msg = ServerMessage::TurnComplete {
                    content: "(cancelled)".to_string(),
                };
                let _ = writeln!(writer, "{}", serde_json::to_string(&cancel_msg).unwrap());
            }
            ClientMessage::Init { .. } => {
                let ready_msg = ServerMessage::SessionReady {
                    profile: bootstrap.client.profile_name.clone(),
                    model: bootstrap.client.profile.model.clone(),
                    pending_proposals: get_pending_proposal_ids(&util::config_dir_str()),
                };
                let _ = writeln!(writer, "{}", serde_json::to_string(&ready_msg).unwrap());
            }
        }
    }

    Ok(())
}

// ── LLM session bootstrap ────────────────────────────────────────────────────

/// Initialises the LLM-powered agent session.
///
/// Returns `None` when `/etc/agntos/models.toml` is missing or invalid, which
/// disables LLM mode and activates keyword-matching fallback.
fn init_llm_session() -> Option<(tokio::runtime::Runtime, LlmSession)> {
    let cfg = util::config_dir_str();
    let runtime = tokio::runtime::Runtime::new().ok()?;
    let client = llm::LlmClient::from_config(&cfg, "chat").ok()?;
    let session_store = SessionStore::from_config_dir(&cfg).ok()?;
    let session_id = SessionStore::new_session_id();
    let inspect_summary = util::capture_inspect("system");
    let _ = agent::seed_memory_if_empty(&cfg, &inspect_summary);
    let system_prompt = llm::build_system_prompt(&cfg, &inspect_summary);
    let system_prompt = inject_prior_context(&system_prompt, &session_store, 8);

    let state = LlmSession {
        client,
        messages: vec![serde_json::json!({
            "role": "system",
            "content": system_prompt,
        })],
        tools: llm::tool_definitions(),
        session_store,
        session_id,
    };

    Some((runtime, state))
}

/// Reads the most recent session turns and appends a compact summary to the
/// system prompt so the agent has continuity across restarts.
fn inject_prior_context(prompt: &str, store: &SessionStore, limit: usize) -> String {
    let turns = match store.recent_turns(limit) {
        Ok(t) => t,
        Err(_) => return prompt.to_string(),
    };

    if turns.is_empty() {
        return prompt.to_string();
    }

    let mut summary = String::from("\n\n## Prior conversation (from last session)\n");
    for turn in turns.iter().rev() {
        let role = match turn.role.as_str() {
            "user" => "User",
            "assistant" => "Agent",
            "tool" => "  Tool",
            _ => &turn.role,
        };
        let snippet: String = turn.content.chars().take(120).collect();
        let ellipsis = if turn.content.len() > 120 { "…" } else { "" };
        summary.push_str(&format!("{}: {}{}\n", role, snippet, ellipsis));
    }

    format!("{}{}", prompt, summary)
}

// ── Status display ───────────────────────────────────────────────────────────

fn print_banner() {
    println!("agntd: AgntOS agent daemon");
    println!("type 'help' for commands, 'quit' to exit.\n");
}

fn print_llm_status(llm_state: &Option<(tokio::runtime::Runtime, LlmSession)>) {
    if let Some((_, state)) = llm_state {
        println!(
            "LLM mode: enabled (task=chat, profile={}, model={})\n",
            state.client.profile_name, state.client.profile.model
        );
    } else {
        println!(
            "LLM mode: disabled (configure /etc/agntos/models.toml or set AGNTOS_CONFIG_DIR)\n"
        );
    }
}

fn show_help(llm_state: &Option<(tokio::runtime::Runtime, LlmSession)>) {
    if llm_state.is_some() {
        println!("  LLM mode active — type anything and the agent will use tools as needed.");
        println!("  Explicit shortcuts still work but are routed through the LLM.\n");
    } else {
        println!("  LLM mode not active — using keyword commands.\n");
    }
    println!("  Commands:");
    println!("  inspect [target]       Inspect system");
    println!("  install <package>      Propose installing a package");
    println!("  remove <package>       Propose removing a package");
    println!("  enable <service>       Propose enabling a service");
    println!("  disable <service>      Propose disabling a service");
    println!("  propose <description>  Propose a custom Nix config change");
    println!("  apply <id>             Apply an approved proposal");
    println!("  audit                  Show recent audit log");
    println!("  audit show <id>        Show audit entry details");
    println!("  rollback               Roll back to previous NixOS generation");
    println!("  history <query>        Search prior conversation turns");
    println!("  help | ?               Show this help");
    println!("  quit | exit | bye      Exit");
}

// ── Keyword-matching fallback (when models.toml is absent) ──────────────────

fn handle_keyword_fallback(lower: &str) {
    if lower.starts_with("inspect ") || lower == "inspect" {
        handle_inspect(lower);
    } else if lower.starts_with("install ") {
        handle_install(lower);
    } else if lower.starts_with("remove ") || lower.starts_with("uninstall ") {
        handle_remove(lower);
    } else if lower.starts_with("enable ") {
        handle_enable(lower);
    } else if lower.starts_with("disable ") {
        handle_disable(lower);
    } else if lower.starts_with("propose ") {
        handle_propose(lower);
    } else if lower.starts_with("apply ") {
        handle_apply(lower);
    } else if lower == "audit" || lower.starts_with("audit ") {
        handle_audit(lower);
    } else if lower == "rollback" || lower.starts_with("rollback ") {
        handle_rollback(lower);
    } else {
        println!("  Unknown command. Type 'help' for available commands.");
        println!("  Tip: configure /etc/agntos/models.toml for LLM-powered interaction.");
    }
}

fn handle_inspect(lower: &str) {
    let target = lower.strip_prefix("inspect ").unwrap_or("system");
    let target = target.split_whitespace().next().unwrap_or("system");
    let stdout = util::capture_inspect(target);
    println!("  Inspects the running system.\n");
    if !stdout.is_empty() {
        println!("{}", stdout);
    }
    println!("\n  (use `audit` to view the activity log for details)");
}

fn handle_install(lower: &str) {
    let package = lower.strip_prefix("install ").unwrap().trim();
    if package.is_empty() {
        println!("  Usage: install <package>");
        return;
    }
    println!("  Proposing installation of: {}", package);
    let output = util::propose_change("install", package);
    if output.is_empty() {
        return;
    }
    println!("{}", output);
    if util::confirm("\n  Apply this proposal?") {
        if let Some(id) = util::extract_proposal_id(&output) {
            util::apply_proposal(&id);
        } else {
            println!("  Could not find proposal ID.");
        }
    }
}

fn handle_remove(lower: &str) {
    let prefix = if lower.starts_with("remove ") {
        "remove "
    } else {
        "uninstall "
    };
    let package = lower.strip_prefix(prefix).unwrap().trim();
    if package.is_empty() {
        println!("  Usage: remove <package>");
        return;
    }
    println!("  Proposing removal of: {}", package);
    let output = util::propose_change("remove", package);
    if output.is_empty() {
        return;
    }
    println!("{}", output);
    if util::confirm("\n  Apply this proposal?") {
        if let Some(id) = util::extract_proposal_id(&output) {
            util::apply_proposal(&id);
        }
    }
}

fn handle_enable(lower: &str) {
    let service = lower.strip_prefix("enable ").unwrap().trim();
    if service.is_empty() {
        println!("  Usage: enable <service>");
        return;
    }
    println!("  Proposing enable of service: {}", service);
    let output = util::propose_change("enable", service);
    if output.is_empty() {
        return;
    }
    println!("{}", output);
    if util::confirm("\n  Apply this proposal?") {
        if let Some(id) = util::extract_proposal_id(&output) {
            util::apply_proposal(&id);
        }
    }
}

fn handle_disable(lower: &str) {
    let service = lower.strip_prefix("disable ").unwrap().trim();
    if service.is_empty() {
        println!("  Usage: disable <service>");
        return;
    }
    println!("  Proposing disable of service: {}", service);
    let output = util::propose_change("disable", service);
    if output.is_empty() {
        return;
    }
    println!("{}", output);
    if util::confirm("\n  Apply this proposal?") {
        if let Some(id) = util::extract_proposal_id(&output) {
            util::apply_proposal(&id);
        }
    }
}

fn handle_propose(lower: &str) {
    let description = lower.strip_prefix("propose ").unwrap_or(lower);
    if description.is_empty() {
        println!("  Usage: propose <description>");
        return;
    }
    let output = util::propose_change("propose", description);
    if output.is_empty() {
        return;
    }
    println!("{}", output);
    if util::confirm("\n  Apply this proposal?") {
        if let Some(id) = util::extract_proposal_id(&output) {
            util::apply_proposal(&id);
        }
    }
}

fn handle_apply(lower: &str) {
    let id = lower
        .strip_prefix("apply ")
        .unwrap_or(lower)
        .split_whitespace()
        .next()
        .unwrap_or("");
    if id.is_empty() {
        println!("  Usage: apply <proposal-id>");
        return;
    }
    if !util::confirm(&format!(
        "  Apply proposal {}? This may change system configuration.",
        id
    )) {
        println!("  Cancelled.");
        return;
    }
    util::apply_proposal(id);
}

fn handle_audit(lower: &str) {
    let parts: Vec<&str> = lower.split_whitespace().collect();
    match parts.as_slice() {
        ["audit"] | ["audit", "list"] => {
            let cfg = util::config_dir_str();
            match util::run_agntctl(&["audit", "list", "--config-dir", &cfg]) {
                Ok((stdout, stderr, success)) => util::print_output(&stdout, &stderr, success),
                Err(e) => println!("  Error: {}", e),
            }
        }
        ["audit", "show", id] => {
            let cfg = util::config_dir_str();
            match util::run_agntctl(&["audit", "show", id, "--config-dir", &cfg]) {
                Ok((stdout, stderr, success)) => util::print_output(&stdout, &stderr, success),
                Err(e) => println!("  Error: {}", e),
            }
        }
        _ => println!("  Usage: audit [list|show <id>]"),
    }
}

fn handle_rollback(lower: &str) {
    let parts: Vec<&str> = lower.split_whitespace().collect();
    let action = parts.get(1).copied().unwrap_or("apply");
    let cfg = util::config_dir_str();

    match action {
        "list" | "list-generations" => {
            match util::run_agntctl(&["rollback", "list", "--config-dir", &cfg]) {
                Ok((stdout, stderr, success)) => util::print_output(&stdout, &stderr, success),
                Err(e) => println!("  Error: {}", e),
            }
        }
        "apply" => {
            if !util::confirm("  Roll back to previous NixOS generation?") {
                println!("  Cancelled.");
                return;
            }
            match util::run_agntctl(&["rollback", "apply", "--config-dir", &cfg]) {
                Ok((stdout, stderr, success)) => util::print_output(&stdout, &stderr, success),
                Err(e) => println!("  Error: {}", e),
            }
        }
        _ => println!("  Usage: rollback [list|apply]"),
    }
}

// ── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_proposal_id() {
        let output = "Proposal: p-abc123\nSummary: Install firefox";
        assert_eq!(
            util::extract_proposal_id(output),
            Some("p-abc123".to_string())
        );
    }

    #[test]
    fn test_extract_proposal_id_no_match() {
        assert_eq!(util::extract_proposal_id("No proposal here"), None);
    }

    #[test]
    fn test_extract_proposal_summary() {
        let out = "Proposal: p-abc\nSummary:  Install curl\n";
        assert_eq!(
            util::extract_proposal_summary(out),
            Some("Install curl".to_string())
        );
    }

    #[test]
    fn test_config_dir_str_default() {
        let dir = util::config_dir_str();
        assert!(!dir.is_empty(), "config dir should not be empty");
    }

    #[test]
    fn test_find_agntctl_returns_something() {
        let path = util::find_agntctl();
        assert!(
            !path.is_empty(),
            "find_agntctl should return a non-empty path"
        );
    }
}
