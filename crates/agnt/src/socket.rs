use agnt_common::wire::{
    AuditRequestAction, ClientMessage, ServerMessage, TokenChannel, ToolCallStatus,
};
use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::UnixStream;
use std::path::Path;
use std::sync::mpsc;
use std::thread;

#[derive(Debug, Clone)]
pub struct SessionInfo {
    pub profile: String,
    pub model: String,
    pub pending_proposals: Vec<String>,
}

#[derive(Debug, Clone)]
pub enum ServerEvent {
    Ready(SessionInfo),
    Token {
        content: String,
        channel: TokenChannel,
    },
    ToolCall {
        name: String,
        status: ToolCallStatus,
        args: serde_json::Value,
    },
    ToolResult {
        name: String,
        success: bool,
        output: String,
    },
    ApprovalRequest {
        proposal_id: String,
        summary: String,
    },
    TurnComplete {
        content: String,
    },
    StatusResponse {
        target: String,
        output: String,
    },
    AuditResponse {
        lines: Vec<String>,
    },
    Error {
        message: String,
    },
}

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

pub struct SocketSession {
    stream: UnixStream,
    events: mpsc::Receiver<ServerEvent>,
}

impl SocketSession {
    pub fn connect(path: &str) -> Result<(Self, SessionInfo), String> {
        let mut stream = UnixStream::connect(path)
            .map_err(|e| format!("cannot connect to {}: {}", path, e))?;
        let mut reader = BufReader::new(stream.try_clone().map_err(|e| e.to_string())?);

        let init = serde_json::to_string(&ClientMessage::Init { config_dir: None })
            .map_err(|e| e.to_string())?;
        writeln!(stream, "{}", init).map_err(|e| e.to_string())?;

        let ready_line = read_line(&mut reader)?;
        let info = parse_ready(&ready_line)?;

        let (tx, rx) = mpsc::channel();
        let read_stream = stream.try_clone().map_err(|e| e.to_string())?;
        thread::spawn(move || {
            let reader = BufReader::new(read_stream);
            for line in reader.lines() {
                let Ok(line) = line else { break };
                if line.trim().is_empty() {
                    continue;
                }
                match parse_event(&line) {
                    Some(ev) => {
                        if tx.send(ev).is_err() {
                            break;
                        }
                    }
                    None => {
                        let _ = tx.send(ServerEvent::Error {
                            message: format!("unrecognized server message: {}", line),
                        });
                    }
                }
            }
            let _ = tx.send(ServerEvent::Error {
                message: "disconnected from agntd".into(),
            });
        });

        Ok((
            Self {
                stream,
                events: rx,
            },
            info,
        ))
    }

    pub fn send_chat(&mut self, prompt: &str) -> Result<(), String> {
        self.send_json(&ClientMessage::Chat {
            prompt: prompt.to_string(),
        })
    }

    pub fn send_cancel(&mut self) -> Result<(), String> {
        self.send_json(&ClientMessage::Cancel)
    }

    pub fn send_init(&mut self) -> Result<(), String> {
        self.send_json(&ClientMessage::Init { config_dir: None })
    }

    pub fn send_status(&mut self, target: &str) -> Result<(), String> {
        self.send_json(&ClientMessage::Status {
            target: target.to_string(),
        })
    }

    pub fn send_audit_list(&mut self, limit: u32) -> Result<(), String> {
        self.send_json(&ClientMessage::Audit {
            action: AuditRequestAction::List,
            query: None,
            id: None,
            limit,
        })
    }

    pub fn send_audit_search(&mut self, query: &str, limit: u32) -> Result<(), String> {
        self.send_json(&ClientMessage::Audit {
            action: AuditRequestAction::Search,
            query: Some(query.to_string()),
            id: None,
            limit,
        })
    }

    pub fn send_audit_show(&mut self, id: &str) -> Result<(), String> {
        self.send_json(&ClientMessage::Audit {
            action: AuditRequestAction::Show,
            query: None,
            id: Some(id.to_string()),
            limit: 20,
        })
    }

    pub fn send_approve(&mut self, proposal_id: &str) -> Result<(), String> {
        self.send_json(&ClientMessage::Approve {
            proposal_id: proposal_id.to_string(),
        })
    }

    pub fn send_dismiss(&mut self, proposal_id: &str, reason: &str) -> Result<(), String> {
        self.send_json(&ClientMessage::Dismiss {
            proposal_id: proposal_id.to_string(),
            reason: Some(reason.to_string()),
        })
    }

    pub fn recv_event(&self) -> Result<ServerEvent, String> {
        self.events
            .recv()
            .map_err(|_| "connection closed".to_string())
    }

    pub fn try_recv_event(&self) -> Option<ServerEvent> {
        self.events.try_recv().ok()
    }

    fn send_json(&mut self, msg: &ClientMessage) -> Result<(), String> {
        let json = serde_json::to_string(msg).map_err(|e| e.to_string())?;
        writeln!(self.stream, "{}", json).map_err(|e| e.to_string())
    }
}

fn read_line(reader: &mut BufReader<UnixStream>) -> Result<String, String> {
    let mut line = String::new();
    reader.read_line(&mut line).map_err(|e| e.to_string())?;
    Ok(line.trim().to_string())
}

fn parse_ready(line: &str) -> Result<SessionInfo, String> {
    let msg: ServerMessage =
        serde_json::from_str(line).map_err(|e| format!("bad session_ready: {}", e))?;
    match msg {
        ServerMessage::SessionReady {
            profile,
            model,
            pending_proposals,
        } => Ok(SessionInfo {
            profile,
            model,
            pending_proposals,
        }),
        ServerMessage::Error { message } => Err(message),
        _ => Err("expected session_ready".into()),
    }
}

fn parse_event(line: &str) -> Option<ServerEvent> {
    let msg: ServerMessage = serde_json::from_str(line).ok()?;
    match msg {
        ServerMessage::SessionReady {
            profile,
            model,
            pending_proposals,
        } => Some(ServerEvent::Ready(SessionInfo {
            profile,
            model,
            pending_proposals,
        })),
        ServerMessage::Token { content, channel } => Some(ServerEvent::Token { content, channel }),
        ServerMessage::ToolCall { name, status, args, .. } => {
            Some(ServerEvent::ToolCall { name, status, args })
        }
        ServerMessage::ToolResult {
            name,
            output,
            success,
            ..
        } => Some(ServerEvent::ToolResult {
            name,
            success,
            output,
        }),
        ServerMessage::ApprovalRequest {
            proposal_id,
            summary,
            ..
        } => Some(ServerEvent::ApprovalRequest {
            proposal_id,
            summary,
        }),
        ServerMessage::TurnComplete { content } => Some(ServerEvent::TurnComplete { content }),
        ServerMessage::StatusResponse { target, data } => {
            let output = data
                .get("output")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            Some(ServerEvent::StatusResponse { target, output })
        }
        ServerMessage::AuditResponse { entries } => {
            let lines: Vec<String> = entries
                .iter()
                .map(|e| serde_json::to_string(e).unwrap_or_else(|_| e.to_string()))
                .collect();
            Some(ServerEvent::AuditResponse { lines })
        }
        ServerMessage::Error { message } => Some(ServerEvent::Error { message }),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_token_event() {
        let line = r#"{"type":"token","content":"hi"}"#;
        match parse_event(line).unwrap() {
            ServerEvent::Token { content, channel } => {
                assert_eq!(content, "hi");
                assert_eq!(channel, TokenChannel::Content);
            }
            _ => panic!("expected token"),
        }
    }
}
