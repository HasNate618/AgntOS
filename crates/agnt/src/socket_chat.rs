use agnt_common::wire::{ClientMessage, ServerMessage};
use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::UnixStream;
use std::path::Path;

pub fn default_socket_path() -> String {
    if let Ok(p) = std::env::var("AGNTOS_SOCKET") {
        if !p.is_empty() {
            return p;
        }
    }
    if let Ok(run) = std::env::var("XDG_RUNTIME_DIR") {
        return format!("{}/agntd.sock", run);
    }
    "/run/agntd/agent.sock".to_string()
}

pub fn socket_available(path: &str) -> bool {
    Path::new(path).exists()
}

pub fn run(socket_path: &str) -> Result<(), String> {
    let mut stream = UnixStream::connect(socket_path)
        .map_err(|e| format!("cannot connect to {}: {}", socket_path, e))?;
    let mut reader = BufReader::new(stream.try_clone().map_err(|e| e.to_string())?);

    let init = serde_json::to_string(&ClientMessage::Init { config_dir: None })
        .map_err(|e| e.to_string())?;
    writeln!(stream, "{}", init).map_err(|e| e.to_string())?;

    let ready = read_line(&mut reader)?;
    let ready_msg: ServerMessage =
        serde_json::from_str(&ready).map_err(|e| format!("bad session_ready: {}", e))?;
    if let ServerMessage::SessionReady {
        profile,
        model,
        pending_proposals,
    } = ready_msg
    {
        println!("agnt: connected (profile={}, model={})", profile, model);
        if !pending_proposals.is_empty() {
            println!("  pending proposals: {}", pending_proposals.join(", "));
        }
    }

    println!("Type a message (/quit to exit).\n");
    let stdin = BufReader::new(std::io::stdin());
    for line in stdin.lines() {
        let line = line.map_err(|e| e.to_string())?;
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        if trimmed == "/quit" || trimmed == "quit" || trimmed == "exit" {
            break;
        }

        let chat = serde_json::to_string(&ClientMessage::Chat {
            prompt: trimmed.to_string(),
        })
        .map_err(|e| e.to_string())?;
        writeln!(stream, "{}", chat).map_err(|e| e.to_string())?;

        loop {
            let msg_line = read_line(&mut reader)?;
            if msg_line.is_empty() {
                continue;
            }
            let msg: ServerMessage =
                serde_json::from_str(&msg_line).map_err(|e| format!("bad server msg: {}", e))?;
            match msg {
                ServerMessage::Token { content } => {
                    print!("{}", content);
                    use std::io::Write as _;
                    let _ = std::io::stdout().flush();
                }
                ServerMessage::ToolCall { name, status, .. } => {
                    println!("\n  [tool] {} ({:?})", name, status);
                }
                ServerMessage::ToolResult { name, output, success, .. } => {
                    let mark = if success { "ok" } else { "fail" };
                    println!("  [tool {}] {}: {}", mark, name, output.lines().next().unwrap_or(""));
                }
                ServerMessage::ApprovalRequest {
                    proposal_id,
                    summary,
                    ..
                } => {
                    println!("\n  [approval] {} — {}", proposal_id, summary);
                    print!("  Apply? [y/N]: ");
                    let _ = std::io::stdout().flush();
                    let mut answer = String::new();
                    std::io::stdin().read_line(&mut answer).map_err(|e| e.to_string())?;
                    let approve = answer.trim().eq_ignore_ascii_case("y");
                    let reply = if approve {
                        ClientMessage::Approve { proposal_id }
                    } else {
                        ClientMessage::Dismiss {
                            proposal_id,
                            reason: Some("declined in TUI".into()),
                        }
                    };
                    let json = serde_json::to_string(&reply).map_err(|e| e.to_string())?;
                    writeln!(stream, "{}", json).map_err(|e| e.to_string())?;
                }
                ServerMessage::TurnComplete { content } => {
                    if !content.is_empty() {
                        println!("\n{}", content);
                    } else {
                        println!();
                    }
                    break;
                }
                ServerMessage::Error { message } => {
                    eprintln!("\n  error: {}", message);
                    break;
                }
                _ => {}
            }
        }
    }

    Ok(())
}

fn read_line(reader: &mut BufReader<UnixStream>) -> Result<String, String> {
    let mut line = String::new();
    reader
        .read_line(&mut line)
        .map_err(|e| e.to_string())?;
    Ok(line.trim().to_string())
}
