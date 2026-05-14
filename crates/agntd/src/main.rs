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
mod llm;
mod session;
mod util;

use agent::LlmSession;
use session::SessionStore;
use std::io::{self, Write};

// ── Entry point ──────────────────────────────────────────────────────────────

fn main() {
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

        // ── Always-local commands ──────────────────────────────────────────
        match lower.as_str() {
            "quit" | "exit" | "bye" => {
                println!("bye.");
                break;
            }
            "help" | "?" => {
                show_help(&llm_state);
                continue;
            }
            _ => {}
        }

        // ── Session-store commands (require LLM session) ───────────────────
        if lower.starts_with("history ") {
            if let Some((runtime, state)) = llm_state.as_mut() {
                let _ = agent::agent_turn(&input, runtime, state);
            } else {
                println!("  LLM mode not active. Configure /etc/agntos/models.toml first.");
            }
            continue;
        }

        // ── Primary path: LLM agent ────────────────────────────────────────
        if let Some((runtime, state)) = llm_state.as_mut() {
            if let Err(e) = agent::agent_turn(&input, runtime, state) {
                println!("  Agent error: {}", e);
            }
            continue;
        }

        // ── Fallback: keyword matching ─────────────────────────────────────
        handle_keyword_fallback(&lower);
    }
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
    let _ = seed_memory_if_empty(&cfg, &inspect_summary);
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

/// Seeds `MEMORY.md` with a compact system snapshot on first run.
/// Does nothing if memory already contains data.
fn seed_memory_if_empty(config_dir: &str, inspect_summary: &str) -> Result<(), String> {
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
