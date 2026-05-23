use crate::socket::{ServerEvent, SocketSession};
use std::io::Write;

pub fn run(socket_path: &str) -> Result<(), String> {
    let (mut session, info) = SocketSession::connect(socket_path)?;
    println!(
        "agnt: connected (profile={}, model={})",
        info.profile, info.model
    );
    if !info.pending_proposals.is_empty() {
        println!("  pending proposals: {}", info.pending_proposals.join(", "));
    }
    println!("Type a message (/help, /quit to exit).\n");

    let stdin = std::io::BufReader::new(std::io::stdin());
    use std::io::BufRead;
    for line in stdin.lines() {
        let line = line.map_err(|e| e.to_string())?;
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        if trimmed == "/quit" || trimmed == "quit" || trimmed == "exit" {
            break;
        }
        if trimmed == "/help" {
            println!("  /help /quit /clear /new /cancel /model /status [target]");
            continue;
        }
        if trimmed == "/clear" {
            println!("  (clear — use TUI for scrollback)");
            continue;
        }
        if trimmed == "/new" {
            session.send_init()?;
            println!("  session refreshed");
            continue;
        }
        if trimmed == "/cancel" {
            session.send_cancel()?;
            continue;
        }
        if trimmed == "/model" {
            println!("  model: {} profile: {}", info.model, info.profile);
            continue;
        }

        session.send_chat(trimmed)?;
        let mut assistant = String::new();
        loop {
            match session.recv_event() {
                Ok(ServerEvent::Token(content)) => {
                    print!("{}", content);
                    assistant.push_str(&content);
                    let _ = std::io::stdout().flush();
                }
                Ok(ServerEvent::ToolCall { name, status, .. }) => {
                    println!("\n  [tool] {} ({:?})", name, status);
                }
                Ok(ServerEvent::ToolResult {
                    name,
                    output,
                    success,
                    ..
                }) => {
                    let mark = if success { "ok" } else { "fail" };
                    println!(
                        "  [tool {}] {}: {}",
                        mark,
                        name,
                        output.lines().next().unwrap_or("")
                    );
                }
                Ok(ServerEvent::ApprovalRequest {
                    proposal_id,
                    summary,
                    ..
                }) => {
                    println!("\n  [approval] {} — {}", proposal_id, summary);
                    print!("  Apply? [y/N]: ");
                    let _ = std::io::stdout().flush();
                    let mut answer = String::new();
                    std::io::stdin().read_line(&mut answer).map_err(|e| e.to_string())?;
                    if answer.trim().eq_ignore_ascii_case("y") {
                        session.send_approve(&proposal_id)?;
                    } else {
                        session.send_dismiss(&proposal_id, "declined")?;
                    }
                }
                Ok(ServerEvent::TurnComplete { content }) => {
                    if assistant.is_empty() && !content.is_empty() {
                        println!("{}", content);
                    } else if !assistant.is_empty() {
                        println!();
                    }
                    break;
                }
                Ok(ServerEvent::Error { message }) => {
                    eprintln!("\n  error: {}", message);
                    break;
                }
                Ok(_) => {}
                Err(e) => {
                    eprintln!("  {}", e);
                    break;
                }
            }
        }
    }
    Ok(())
}
