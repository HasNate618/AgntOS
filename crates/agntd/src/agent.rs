//! LLM-powered agent turn: receives one user input, runs the tool-calling loop against
//! the configured endpoint, persists session turns, and displays results.
//!
//! ## Flow
//! 1. The user's text is appended to the conversation as a `"user"` message.
//! 2. The LLM is called with the full message history and the AgntOS tool definitions.
//! 3. If the LLM returns tool calls, each is executed via `agntctl` (with interactive
//!    confirmation for the `apply` tool).  Tool results are fed back as `"tool"` messages
//!    and the loop repeats up to a depth limit.
//! 4. When the LLM returns a plain text response it is printed and the turn ends.
//!
//! ## Socket / daemon mode
//! When `agntd` is started with `--socket <path>` it listens on a Unix domain socket
//! for one-shot JSON requests instead of running a REPL.  Each connection sends a
//! `{"prompt": "..."}` and receives a `{"response": "..."}` JSON reply.  The daemon
//! shares the same agent loop but creates a fresh conversation per request.

use crate::llm::{LlmClient, ToolCall};
use crate::session::SessionStore;
use crate::util;
use serde_json::{json, Value};
use std::sync::{Arc, Mutex};

/// Approval gate for GUI-driven confirmations. The persistent session handler
/// sets resolved/approved when the user responds via Approve/Dismiss messages.
pub struct ApprovalGate {
    pub proposal_id: String,
    #[allow(dead_code)]
    pub tool_call_id: String,
    #[allow(dead_code)]
    pub summary: String,
    pub resolved: bool,
    pub approved: bool,
}

pub type SharedApprovalGate = Arc<Mutex<Option<ApprovalGate>>>;

/// Encapsulates the mutable state that an LLM-powered agent session needs.
pub struct LlmSession {
    pub client: LlmClient,
    pub messages: Vec<Value>,
    pub tools: Vec<Value>,
    pub session_store: SessionStore,
    pub session_id: String,
}

// ── Daemon / Socket mode ───────────────────────────────────────────────────

/// Shared state held by the socket-mode daemon, reused across connections.
pub struct DaemonBootstrap {
    pub runtime: tokio::runtime::Runtime,
    pub client: LlmClient,
    pub tools: Vec<Value>,
    pub session_store: SessionStore,
}

impl DaemonBootstrap {
    /// Creates a bootstrap from the config directory, loading the LLM client,
    /// session store, and tool definitions. Seeds memory if empty.
    pub fn from_config_dir(config_dir: &str) -> Result<Self, String> {
        let runtime = tokio::runtime::Runtime::new()
            .map_err(|e| format!("Failed to create runtime: {}", e))?;
        let client = LlmClient::from_config(config_dir, "chat")?;
        let session_store = SessionStore::from_config_dir(config_dir)?;
        let tools = crate::llm::tool_definitions();
        let inspect_summary = util::capture_inspect("system");
        let _ = seed_memory_if_empty(config_dir, &inspect_summary);
        Ok(Self {
            runtime,
            client,
            tools,
            session_store,
        })
    }
}

/// Processes one prompt through the LLM tool-calling loop and returns the
/// final response text.  Unlike the REPL [`agent_turn`], this does NOT print
/// output or accumulate messages across calls — it is a stateless one-shot.
///
/// Confirmation-gated tools (`apply`, `rollback`) will be cancelled when
/// stdin is not a terminal (as is the case in socket mode), because the
/// underlying `util::confirm()` reads from stdin and returns `false` on EOF.
pub fn process_prompt(input: &str, bootstrap: &DaemonBootstrap) -> Result<String, String> {
    let inspect_summary = util::capture_inspect("system");
    let system_prompt = crate::llm::build_system_prompt(&util::config_dir_str(), &inspect_summary);
    let mut messages: Vec<Value> = vec![
        json!({"role": "system", "content": system_prompt}),
        json!({"role": "user", "content": input}),
    ];

    let mut depth = 0;
    while depth < 8 {
        depth += 1;
        let resp = bootstrap
            .runtime
            .block_on(bootstrap.client.complete(&messages, &bootstrap.tools))?;
        messages.push(resp.assistant_message.clone());

        if resp.tool_calls.is_empty() {
            let content = resp.content;
            // Persist
            let sid = SessionStore::new_session_id();
            let _ = bootstrap
                .session_store
                .append_turn(&sid, "user", input, None);
            let _ = bootstrap
                .session_store
                .append_turn(&sid, "assistant", &content, None);
            return Ok(content);
        }

        for tc in &resp.tool_calls {
            let result = match execute_tool_call(tc, Some(input)) {
                Ok(s) => s,
                Err(e) => format!("TOOL_ERROR: {}", e),
            };
            messages.push(json!({
                "role": "tool",
                "tool_call_id": tc.id,
                "content": result,
            }));
        }
    }

    Err("Tool-call depth limit reached".to_string())
}

/// Processes one user input through the LLM tool-calling loop.
///
/// `history` commands are intercepted before any LLM call so that they work even when
/// the endpoint is unreachable.
pub fn agent_turn(
    input: &str,
    runtime: &mut tokio::runtime::Runtime,
    state: &mut LlmSession,
) -> Result<(), String> {
    let _ = state
        .session_store
        .append_turn(&state.session_id, "user", input, None);

    // Built-in commands that use the local session store.
    if let Some(query) = input.strip_prefix("history ") {
        return handle_history_command(query.trim(), state);
    }

    state.messages.push(json!({
        "role": "user",
        "content": input,
    }));

    let mut depth = 0;
    while depth < 8 {
        depth += 1;
        let resp = runtime.block_on(
            state
                .client
                .complete_streaming(&state.messages, &state.tools),
        )?;
        state.messages.push(resp.assistant_message.clone());

        // Plain text response — content was already streamed to stdout.
        if resp.tool_calls.is_empty() {
            if !resp.content.trim().is_empty() {
                let _ = state.session_store.append_turn(
                    &state.session_id,
                    "assistant",
                    &resp.content,
                    None,
                );
            } else {
                println!("  (no response text)");
            }
            return Ok(());
        }

        // Execute each tool call and feed results back to the LLM.
        for tc in &resp.tool_calls {
            let result = match execute_tool_call(tc, Some(input)) {
                Ok(s) => s,
                Err(e) => format!("TOOL_ERROR: {}", e),
            };
            let _ =
                state
                    .session_store
                    .append_turn(&state.session_id, "tool", &result, Some(&tc.name));
            state.messages.push(json!({
                "role": "tool",
                "tool_call_id": tc.id,
                "content": result,
            }));
        }
    }

    Err("Tool-call depth limit reached".to_string())
}

/// Executes a single tool call by delegating to `agntctl` (or the in-process memory module).
///
/// The `apply` tool requires interactive user confirmation before proceeding.
fn execute_tool_call(tc: &ToolCall, user_prompt: Option<&str>) -> Result<String, String> {
    let cfg = util::config_dir_str();
    let args = tc.arguments.as_object().cloned().unwrap_or_default();

    match tc.name.as_str() {
        "inspect" => {
            let target = args
                .get("target")
                .and_then(|v| v.as_str())
                .unwrap_or("system");
            command_result(util::run_agntctl(&["inspect", target]))
        }
        "propose" => {
            let description = args
                .get("description")
                .and_then(|v| v.as_str())
                .ok_or_else(|| "Missing required argument: description".to_string())?;
            let rationale = args.get("rationale").and_then(|v| v.as_str()).unwrap_or("");
            let mut cmd = vec!["propose", "--config-dir", &cfg, description];
            if let Some(p) = user_prompt {
                cmd.push("--prompt");
                cmd.push(p);
            }
            if !rationale.is_empty() {
                cmd.push("--rationale");
                cmd.push(rationale);
            }
            command_result(util::run_agntctl(&cmd))
        }
        "apply" => {
            let proposal_id = args
                .get("proposal_id")
                .and_then(|v| v.as_str())
                .ok_or_else(|| "Missing required argument: proposal_id".to_string())?;

            if !util::confirm(&format!("  LLM requests apply {}. Proceed?", proposal_id)) {
                // Return a message that the LLM can act on: does NOT retry.
                return Ok(format!(
                    "APPLY_CANCELLED: confirmation declined. Proposal {} was NOT applied. \
Do NOT retry — tell the user to run 'agntctl apply {}' if they want to proceed.",
                    proposal_id, proposal_id
                ));
            }

            let no_rebuild = args
                .get("no_rebuild")
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            if no_rebuild {
                command_result(util::run_agntctl(&[
                    "apply",
                    "--no-rebuild",
                    "--config-dir",
                    &cfg,
                    proposal_id,
                ]))
            } else {
                command_result(util::run_agntctl(&[
                    "apply",
                    "--config-dir",
                    &cfg,
                    proposal_id,
                ]))
            }
        }
        "audit" => {
            let action = args
                .get("action")
                .and_then(|v| v.as_str())
                .unwrap_or("list");
            match action {
                "show" => {
                    let id = args.get("id").and_then(|v| v.as_str()).ok_or_else(|| {
                        "Missing required argument for audit show: id".to_string()
                    })?;
                    command_result(util::run_agntctl(&[
                        "audit",
                        "show",
                        id,
                        "--config-dir",
                        &cfg,
                    ]))
                }
                "search" => {
                    let query = args.get("query").and_then(|v| v.as_str()).ok_or_else(|| {
                        "Missing required argument for audit search: query".to_string()
                    })?;
                    let limit_arg = args
                        .get("limit")
                        .and_then(|v| v.as_u64())
                        .map(|n| n.to_string())
                        .unwrap_or_else(|| "20".to_string());
                    command_result(util::run_agntctl(&[
                        "audit",
                        "search",
                        "--query",
                        query,
                        "--limit",
                        &limit_arg,
                        "--config-dir",
                        &cfg,
                    ]))
                }
                _ => {
                    let limit_arg = args
                        .get("limit")
                        .and_then(|v| v.as_u64())
                        .map(|n| n.to_string())
                        .unwrap_or_else(|| "20".to_string());
                    command_result(util::run_agntctl(&[
                        "audit",
                        "list",
                        "--limit",
                        &limit_arg,
                        "--config-dir",
                        &cfg,
                    ]))
                }
            }
        }
        "memory" => {
            let action = args
                .get("action")
                .and_then(|v| v.as_str())
                .unwrap_or("show");
            match action {
                "show" => {
                    let file = args.get("file").and_then(|v| v.as_str());
                    match file {
                        Some(f) => command_result(util::run_agntctl(&[
                            "memory",
                            "show",
                            f,
                            "--config-dir",
                            &cfg,
                        ])),
                        None => command_result(util::run_agntctl(&[
                            "memory",
                            "show",
                            "--config-dir",
                            &cfg,
                        ])),
                    }
                }
                "add" => {
                    let file = args
                        .get("file")
                        .and_then(|v| v.as_str())
                        .ok_or_else(|| "memory add requires file".to_string())?;
                    let section = args
                        .get("section")
                        .and_then(|v| v.as_str())
                        .ok_or_else(|| "memory add requires section".to_string())?;
                    let content = args
                        .get("content")
                        .and_then(|v| v.as_str())
                        .ok_or_else(|| "memory add requires content".to_string())?;
                    command_result(util::run_agntctl(&[
                        "memory",
                        "add",
                        file,
                        "--section",
                        section,
                        "--content",
                        content,
                        "--config-dir",
                        &cfg,
                    ]))
                }
                "replace" => {
                    let file = args
                        .get("file")
                        .and_then(|v| v.as_str())
                        .ok_or_else(|| "memory replace requires file".to_string())?;
                    let target = args
                        .get("target")
                        .and_then(|v| v.as_str())
                        .ok_or_else(|| "memory replace requires target".to_string())?;
                    let replacement = args
                        .get("replacement")
                        .and_then(|v| v.as_str())
                        .ok_or_else(|| "memory replace requires replacement".to_string())?;
                    command_result(util::run_agntctl(&[
                        "memory",
                        "replace",
                        file,
                        "--target",
                        target,
                        "--replacement",
                        replacement,
                        "--config-dir",
                        &cfg,
                    ]))
                }
                "remove" => {
                    let file = args
                        .get("file")
                        .and_then(|v| v.as_str())
                        .ok_or_else(|| "memory remove requires file".to_string())?;
                    let target = args
                        .get("target")
                        .and_then(|v| v.as_str())
                        .ok_or_else(|| "memory remove requires target".to_string())?;
                    command_result(util::run_agntctl(&[
                        "memory",
                        "remove",
                        file,
                        "--target",
                        target,
                        "--config-dir",
                        &cfg,
                    ]))
                }
                "consolidate" => {
                    let file = args
                        .get("file")
                        .and_then(|v| v.as_str())
                        .unwrap_or("memory");
                    command_result(util::run_agntctl(&[
                        "memory",
                        "consolidate",
                        file,
                        "--config-dir",
                        &cfg,
                    ]))
                }
                other => Err(format!("Unsupported memory action: {}", other)),
            }
        }
        "rollback" => {
            if !util::confirm("  LLM requests system rollback. Proceed?") {
                return Ok(
                    "ROLLBACK_CANCELLED: confirmation declined. System was NOT rolled back. \
Do NOT retry — tell the user to run 'agntctl rollback apply' if they want to proceed."
                        .to_string(),
                );
            }
            let audit_id = args.get("audit_id").and_then(|v| v.as_str());
            let mut cmd = vec!["rollback", "--config-dir", &cfg];
            match audit_id {
                Some(id) if !id.is_empty() => {
                    cmd.push("undo");
                    cmd.push("--undo-id");
                    cmd.push(id);
                }
                _ => {
                    cmd.push("apply");
                }
            }
            command_result(util::run_agntctl(&cmd))
        }
        "read_file" => {
            let path = args
                .get("path")
                .and_then(|v| v.as_str())
                .ok_or_else(|| "Missing required argument: path".to_string())?;
            command_result(util::run_agntctl(&["read", path]))
        }
        "write_file" => {
            let path = args
                .get("path")
                .and_then(|v| v.as_str())
                .ok_or_else(|| "Missing required argument: path".to_string())?;
            let content = args
                .get("content")
                .and_then(|v| v.as_str())
                .ok_or_else(|| "Missing required argument: content".to_string())?;
            command_result(util::run_agntctl(&[
                "write",
                path,
                "--content",
                content,
                "--config-dir",
                &cfg,
            ]))
        }
        "edit_file" => {
            let path = args
                .get("path")
                .and_then(|v| v.as_str())
                .ok_or_else(|| "Missing required argument: path".to_string())?;
            let old = args
                .get("old_string")
                .and_then(|v| v.as_str())
                .ok_or_else(|| "Missing required argument: old_string".to_string())?;
            let new = args
                .get("new_string")
                .and_then(|v| v.as_str())
                .ok_or_else(|| "Missing required argument: new_string".to_string())?;
            command_result(util::run_agntctl(&[
                "edit",
                path,
                "--old",
                old,
                "--new",
                new,
                "--config-dir",
                &cfg,
            ]))
        }
        "run_bash" => {
            let command = args
                .get("command")
                .and_then(|v| v.as_str())
                .ok_or_else(|| "Missing required argument: command".to_string())?;
            command_result(util::run_agntctl(&["bash", command, "--config-dir", &cfg]))
        }
        other => Err(format!("Unknown tool: {}", other)),
    }
}

/// Like execute_tool_call but uses an ApprovalGate instead of stdin for confirmations.
/// Used by the persistent session handler to gate apply/rollback via GUI messages.
pub fn execute_tool_call_gui(
    tc: &ToolCall,
    user_prompt: Option<&str>,
    approval_gate: SharedApprovalGate,
) -> Result<String, String> {
    let cfg = util::config_dir_str();
    let args = tc.arguments.as_object().cloned().unwrap_or_default();

    match tc.name.as_str() {
        "apply" => {
            let proposal_id = args
                .get("proposal_id")
                .and_then(|v| v.as_str())
                .ok_or_else(|| "Missing required argument: proposal_id".to_string())?;

            {
                let mut gate = approval_gate.lock().unwrap();
                *gate = Some(ApprovalGate {
                    proposal_id: proposal_id.to_string(),
                    tool_call_id: tc.id.clone(),
                    summary: format!("Apply proposal {}", proposal_id),
                    resolved: false,
                    approved: false,
                });
            }

            let start = std::time::Instant::now();
            loop {
                std::thread::sleep(std::time::Duration::from_millis(50));
                if start.elapsed().as_secs() > 300 {
                    return Err("GUI approval timeout (5 min) — proposal not applied.".to_string());
                }
                let gate = approval_gate.lock().unwrap();
                if let Some(ref g) = *gate {
                    if g.resolved {
                        if g.approved {
                            break;
                        } else {
                            return Ok(format!(
                                "APPLICATION_REJECTED: User declined proposal {}. Do NOT retry.",
                                proposal_id
                            ));
                        }
                    }
                }
            }

            let no_rebuild = args
                .get("no_rebuild")
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            if no_rebuild {
                command_result(util::run_agntctl(&[
                    "apply",
                    "--no-rebuild",
                    "--config-dir",
                    &cfg,
                    proposal_id,
                ]))
            } else {
                command_result(util::run_agntctl(&[
                    "apply",
                    "--config-dir",
                    &cfg,
                    proposal_id,
                ]))
            }
        }
        "rollback" => {
            {
                let mut gate = approval_gate.lock().unwrap();
                *gate = Some(ApprovalGate {
                    proposal_id: String::new(),
                    tool_call_id: tc.id.clone(),
                    summary: "Roll back to previous NixOS generation".to_string(),
                    resolved: false,
                    approved: false,
                });
            }

            let start = std::time::Instant::now();
            loop {
                std::thread::sleep(std::time::Duration::from_millis(50));
                if start.elapsed().as_secs() > 300 {
                    return Err(
                        "GUI approval timeout (5 min) — rollback not performed.".to_string()
                    );
                }
                let gate = approval_gate.lock().unwrap();
                if let Some(ref g) = *gate {
                    if g.resolved {
                        if g.approved {
                            break;
                        } else {
                            return Ok("ROLLBACK_REJECTED: User declined rollback. Do NOT retry."
                                .to_string());
                        }
                    }
                }
            }

            let audit_id = args.get("audit_id").and_then(|v| v.as_str());
            let mut cmd = vec!["rollback", "--config-dir", &cfg];
            match audit_id {
                Some(id) if !id.is_empty() => {
                    cmd.push("undo");
                    cmd.push("--undo-id");
                    cmd.push(id);
                }
                _ => {
                    cmd.push("apply");
                }
            }
            command_result(util::run_agntctl(&cmd))
        }
        _other => execute_tool_call(tc, user_prompt),
    }
}

/// Reads pending proposal IDs from the proposals directory for the session_ready message.
pub fn get_pending_proposal_ids(config_dir: &str) -> Vec<String> {
    let dir = format!("{}/proposals", config_dir);
    let mut ids = Vec::new();
    if let Ok(entries) = std::fs::read_dir(&dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().map_or(false, |e| e == "json") {
                if let Ok(raw) = std::fs::read_to_string(&path) {
                    if let Ok(v) = serde_json::from_str::<serde_json::Value>(&raw) {
                        if let Some(id) = v.get("id").and_then(|i| i.as_str()) {
                            ids.push(id.to_string());
                        }
                    }
                }
            }
        }
    }
    ids
}

/// Wraps the (stdout, stderr, success) tuple from `agntctl` into a single string
/// suitable for feeding back to the LLM as a tool result.  Errors become `Err` so
/// the LLM sees `TOOL_ERROR:` instead of a success string.
fn command_result(cmd: Result<(String, String, bool), String>) -> Result<String, String> {
    match cmd {
        Ok((stdout, stderr, success)) => {
            let mut text = String::new();
            if !stdout.trim().is_empty() {
                text.push_str(stdout.trim());
            }
            if !stderr.trim().is_empty() {
                if !text.is_empty() {
                    text.push_str("\n");
                }
                text.push_str("STDERR:\n");
                text.push_str(stderr.trim());
            }
            if text.is_empty() {
                text.push_str("(no output)");
            }
            // Truncate long results to avoid context overflow
            const MAX_TOOL_RESULT: usize = 2000;
            if text.len() > MAX_TOOL_RESULT {
                text = text.chars().take(MAX_TOOL_RESULT).collect::<String>();
                text.push_str("\n... (output truncated)");
            }
            if success {
                Ok(text)
            } else {
                Err(text)
            }
        }
        Err(e) => Err(e),
    }
}

/// Reviews recent session turns and extracts memory-worthy facts via the LLM.
/// Called at end-of-session (REPL exit) to capture preferences and intent.
/// Falls back to silent consolidation if LLM is unavailable or errors.
pub fn end_of_session_review(
    runtime: &tokio::runtime::Runtime,
    client: &LlmClient,
    session_store: &SessionStore,
) -> Result<String, String> {
    let turns = session_store.recent_turns(30)?;
    if turns.is_empty() {
        return Ok("No session turns to review.".to_string());
    }

    let mut conversation = String::new();
    for turn in &turns {
        if turn.role == "tool" {
            continue;
        }
        let role = match turn.role.as_str() {
            "user" => "User",
            "assistant" => "Agent",
            _ => continue,
        };
        let snippet: String = turn.content.chars().take(250).collect();
        conversation.push_str(&format!("{}: {}\n", role, snippet));
    }

    if conversation.trim().is_empty() {
        return Ok("No user/assistant turns to review.".to_string());
    }

    let prompt = format!(
        "You are a memory curator for an OS agent. Review this conversation and extract \
facts worth remembering for future sessions.\n\n\
Store ONLY:\n\
- User preferences (editor, workflow, naming conventions, config style)\n\
- User intent and long-term goals\n\
- Non-obvious system context not re-derivable from inspect\n\n\
Do NOT store:\n\
- Facts re-derivable via inspect (CPU, RAM, packages, disk usage, services)\n\
- One-time transient information\n\
- Tool outputs or file contents\n\n\
Output a JSON object with two optional arrays:\n\
- \"memory\": list of strings to add to MEMORY.md (system facts)\n\
- \"user\": list of strings to add to USER.md (preferences)\n\
Output empty arrays if nothing worth remembering.\n\n\
Conversation:\n{}\n\n\
JSON:",
        conversation
    );

    let messages = vec![
        serde_json::json!({"role": "system", "content": "You extract memory facts. Be concise. Respond with JSON only."}),
        serde_json::json!({"role": "user", "content": prompt}),
    ];

    let tools: Vec<Value> = vec![];
    let response = runtime
        .block_on(client.complete(&messages, &tools))
        .map_err(|e| format!("Memory review LLM call failed: {}", e))?;

    let response_text = response.content;
    let parsed: Result<serde_json::Value, _> = serde_json::from_str(&response_text);
    let facts = match parsed {
        Ok(v) => v,
        Err(_) => {
            let start = response_text.find('{');
            let end = response_text.rfind('}');
            match (start, end) {
                (Some(s), Some(e)) if e > s => {
                    serde_json::from_str(&response_text[s..=e]).unwrap_or(serde_json::json!({}))
                }
                _ => serde_json::json!({}),
            }
        }
    };

    let mut extracted = 0;
    let cfg = util::config_dir_str();

    if let Some(memory_items) = facts.get("memory").and_then(|a| a.as_array()) {
        for item in memory_items {
            if let Some(text) = item.as_str() {
                let _ = util::run_agntctl(&[
                    "memory",
                    "add",
                    "memory",
                    "--section",
                    "Session",
                    "--content",
                    text,
                    "--config-dir",
                    &cfg,
                ]);
                extracted += 1;
            }
        }
    }

    if let Some(user_items) = facts.get("user").and_then(|a| a.as_array()) {
        for item in user_items {
            if let Some(text) = item.as_str() {
                let _ = util::run_agntctl(&[
                    "memory",
                    "add",
                    "user",
                    "--section",
                    "Session",
                    "--content",
                    text,
                    "--config-dir",
                    &cfg,
                ]);
                extracted += 1;
            }
        }
    }

    let _ = util::run_agntctl(&["memory", "consolidate", "memory", "--config-dir", &cfg]);
    let _ = util::run_agntctl(&["memory", "consolidate", "user", "--config-dir", &cfg]);

    if extracted > 0 {
        Ok(format!(
            "Memory review complete. Extracted {} facts.",
            extracted
        ))
    } else {
        Ok("Memory review complete. Nothing new to store.".to_string())
    }
}

/// Seeds `MEMORY.md` with a compact system snapshot on first run.
/// Does nothing if memory already contains data.
pub fn seed_memory_if_empty(config_dir: &str, inspect_summary: &str) -> Result<(), String> {
    let mut mem = agnt_common::memory::CoreMemory::load(config_dir)?;
    if !mem.memory.trim().is_empty() {
        return Ok(());
    }

    let compact = inspect_summary
        .lines()
        .map(str::trim)
        .filter(|l| !l.is_empty())
        .take(8)
        .collect::<Vec<_>>()
        .join(" | ");

    if compact.is_empty() {
        return Ok(());
    }

    let seeded = if compact.chars().count() > 280 {
        format!("{}...", compact.chars().take(277).collect::<String>())
    } else {
        compact
    };

    mem.add(agnt_common::memory::MemoryFile::Memory, "System", &seeded)
}

/// Searches the local FTS5 session store and prints matching turns.
fn handle_history_command(query: &str, state: &LlmSession) -> Result<(), String> {
    if query.is_empty() {
        println!("  Usage: history <query>");
        return Ok(());
    }

    let hits = state.session_store.search(query, 5)?;
    if hits.is_empty() {
        println!("  No history matches for '{}'.", query);
        return Ok(());
    }

    println!("  History matches for '{}':", query);
    for hit in hits {
        println!(
            "  - [{}] {} {} {}: {}",
            hit.row_id,
            hit.timestamp.format("%Y-%m-%d %H:%M:%S"),
            hit.session_id,
            hit.role,
            hit.content.replace('\n', " ")
        );
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn approval_gate_default_is_none() {
        let gate: SharedApprovalGate = Arc::new(Mutex::new(None));
        let g = gate.lock().unwrap();
        assert!(g.is_none());
    }

    #[test]
    fn approval_gate_set_and_resolve() {
        let gate: SharedApprovalGate = Arc::new(Mutex::new(None));
        {
            let mut g = gate.lock().unwrap();
            *g = Some(ApprovalGate {
                proposal_id: "p-test".to_string(),
                tool_call_id: "tc-1".to_string(),
                summary: "Test".to_string(),
                resolved: false,
                approved: false,
            });
        }
        {
            let mut g = gate.lock().unwrap();
            let g_ref = g.as_mut().unwrap();
            g_ref.resolved = true;
            g_ref.approved = true;
        }
        {
            let g = gate.lock().unwrap();
            assert!(g.as_ref().unwrap().resolved);
            assert!(g.as_ref().unwrap().approved);
        }
    }

    #[test]
    fn get_pending_proposal_ids_empty_when_no_dir() {
        let tmp = std::env::temp_dir().join("agntos-test-proposals-none");
        let _ = std::fs::remove_dir_all(&tmp);
        let ids = get_pending_proposal_ids(&tmp.to_string_lossy());
        assert!(ids.is_empty());
    }

    #[test]
    fn get_pending_proposal_ids_reads_valid_proposals() {
        let tmp = std::env::temp_dir().join("agntos-test-proposals-valid");
        let _ = std::fs::remove_dir_all(&tmp);
        std::fs::create_dir_all(tmp.join("proposals")).unwrap();
        std::fs::write(
            tmp.join("proposals/p-abc123.json"),
            r#"{"id":"p-abc123","summary":"test"}"#,
        )
        .unwrap();
        std::fs::write(
            tmp.join("proposals/p-def456.json"),
            r#"{"id":"p-def456","summary":"test2"}"#,
        )
        .unwrap();
        let ids = get_pending_proposal_ids(&tmp.to_string_lossy());
        assert_eq!(ids.len(), 2);
        assert!(ids.contains(&"p-abc123".to_string()));
        assert!(ids.contains(&"p-def456".to_string()));
        let _ = std::fs::remove_dir_all(&tmp);
    }

    #[test]
    fn get_pending_proposal_ids_skips_invalid_json() {
        let tmp = std::env::temp_dir().join("agntos-test-proposals-invalid");
        let _ = std::fs::remove_dir_all(&tmp);
        std::fs::create_dir_all(tmp.join("proposals")).unwrap();
        std::fs::write(tmp.join("proposals/p-bad.json"), r#"not valid json"#).unwrap();
        std::fs::write(tmp.join("proposals/p-good.json"), r#"{"id":"p-good"}"#).unwrap();
        let ids = get_pending_proposal_ids(&tmp.to_string_lossy());
        assert_eq!(ids.len(), 1);
        assert_eq!(ids[0], "p-good");
        let _ = std::fs::remove_dir_all(&tmp);
    }
}
