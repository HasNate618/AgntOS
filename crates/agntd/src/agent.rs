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
            command_result(util::run_agntctl(&["rollback", "--config-dir", &cfg]))
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
