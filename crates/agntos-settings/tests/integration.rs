use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::{UnixListener, UnixStream};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use agnt_common::wire::{AuditRequestAction, ClientMessage, ServerMessage, ToolCallStatus};
use agntos_settings::backend::session::Connection;
use agntos_settings::session::AppSession;

fn start_mock_server(path: &str, done: Arc<Mutex<bool>>) -> thread::JoinHandle<()> {
    let _ = std::fs::remove_file(path);
    let path = path.to_string();
    thread::spawn(move || {
        let listener = UnixListener::bind(&path).unwrap();
        listener.set_nonblocking(true).ok();

        loop {
            if *done.lock().unwrap() {
                break;
            }

            let stream = match listener.accept() {
                Ok((s, _)) => s,
                Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                    thread::sleep(Duration::from_millis(50));
                    continue;
                }
                Err(_) => break,
            };

            let done_clone = done.clone();
            thread::spawn(move || {
                handle_connection(stream, done_clone);
            });
        }
    })
}

fn handle_connection(stream: UnixStream, _done: Arc<Mutex<bool>>) {
    let mut writer = stream.try_clone().expect("clone stream for writer");
    let mut reader = BufReader::new(stream);

    let mut line = String::new();
    reader.read_line(&mut line).ok();
    line = line.trim().to_string();

    let parsed: ClientMessage = serde_json::from_str(&line).unwrap_or(ClientMessage::Cancel);
    match parsed {
        ClientMessage::Init { .. } => {
            let ready = ServerMessage::SessionReady {
                profile: "test-profile".to_string(),
                model: "test-model".to_string(),
                pending_proposals: vec!["p-test".to_string()],
            };
            writeln!(writer, "{}", serde_json::to_string(&ready).unwrap()).ok();
        }
        _ => return,
    }

    let mut line = String::new();
    reader.read_line(&mut line).ok();
    line = line.trim().to_string();
    let parsed: ClientMessage = serde_json::from_str(&line).unwrap_or(ClientMessage::Cancel);

    match parsed {
        ClientMessage::Status { target } => {
            let resp = ServerMessage::StatusResponse {
                target: target.clone(),
                data: serde_json::json!({"output": "CPU: 8 cores\nRAM: 32 GB"}),
            };
            writeln!(writer, "{}", serde_json::to_string(&resp).unwrap()).ok();
        }
        ClientMessage::Audit { .. } => {
            let resp = ServerMessage::AuditResponse {
                entries: vec![serde_json::json!({
                    "id": "a-001",
                    "timestamp": "2025-05-16T14:30:00Z",
                    "action": {"type": "Apply", "proposal_id": "p-abc"},
                    "summary": "Applied: Install nginx",
                    "result": {"status": "Success", "message": "Rebuild ok"},
                    "actor": "agent",
                    "prompt": "install nginx",
                })],
            };
            writeln!(writer, "{}", serde_json::to_string(&resp).unwrap()).ok();
        }
        ClientMessage::Chat { .. } => {
            let tc_msg = ServerMessage::ToolCall {
                id: "tc-1".to_string(),
                name: "inspect".to_string(),
                args: serde_json::json!({"target": "system"}),
                status: ToolCallStatus::Running,
            };
            writeln!(writer, "{}", serde_json::to_string(&tc_msg).unwrap()).ok();

            let tr_msg = ServerMessage::ToolResult {
                id: "tc-1".to_string(),
                name: "inspect".to_string(),
                output: "CPU: 8 cores".to_string(),
                success: true,
            };
            writeln!(writer, "{}", serde_json::to_string(&tr_msg).unwrap()).ok();

            let done = ServerMessage::TurnComplete {
                content: "Inspection complete.".to_string(),
            };
            writeln!(writer, "{}", serde_json::to_string(&done).unwrap()).ok();
        }
        _ => {}
    }
}

fn serve_mock(path: &str, done: Arc<Mutex<bool>>) -> thread::JoinHandle<()> {
    let _ = std::fs::remove_file(path);
    let path = path.to_string();
    thread::spawn(move || {
        let listener = UnixListener::bind(&path).unwrap();
        listener.set_nonblocking(true).ok();

        let session_stream = loop {
            if *done.lock().unwrap() {
                return;
            }
            match listener.accept() {
                Ok((s, _)) => break s,
                Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                    thread::sleep(Duration::from_millis(50));
                    continue;
                }
                Err(_) => return,
            }
        };

        session_stream.set_read_timeout(Some(Duration::from_millis(200))).ok();
        let mut session_writer = session_stream.try_clone().expect("clone session writer");
        let mut session_reader = BufReader::new(session_stream);

        let mut line = String::new();
        match session_reader.read_line(&mut line) {
            Ok(0) => { return; }
            Ok(_) => {}
            Err(_) => { return; }
        }
        line = line.trim().to_string();
        let parsed: ClientMessage = serde_json::from_str(&line).unwrap_or(ClientMessage::Cancel);
        match parsed {
            ClientMessage::Init { .. } => {
                let ready = ServerMessage::SessionReady {
                    profile: "test-profile".to_string(),
                    model: "test-model".to_string(),
                    pending_proposals: vec![],
                };
                writeln!(session_writer, "{}", serde_json::to_string(&ready).unwrap()).ok();
                session_writer.flush().ok();
            }
            _ => return,
        }

        let script_stream = loop {
            if *done.lock().unwrap() {
                return;
            }
            match listener.accept() {
                Ok((s, _)) => break s,
                Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                    thread::sleep(Duration::from_millis(50));
                    continue;
                }
                Err(_) => return,
            }
        };

        script_stream.set_read_timeout(Some(Duration::from_millis(100))).ok();
        let mut script_reader = BufReader::new(script_stream);

        loop {
            if *done.lock().unwrap() {
                break;
            }

            let mut script_line = String::new();
            match script_reader.read_line(&mut script_line) {
                Ok(0) => { return; }
                Ok(_) => {}
                Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock || e.kind() == std::io::ErrorKind::TimedOut => {
                    thread::sleep(Duration::from_millis(50));
                    continue;
                }
                Err(_) => { return; }
            }
            let script_line = script_line.trim().to_string();
            if script_line.is_empty() {
                return;
            }

            let responses: Vec<ServerMessage> = match serde_json::from_str(&script_line) {
                Ok(r) => r,
                Err(_) => { return; }
            };

            let mut session_line = String::new();
            match session_reader.read_line(&mut session_line) {
                Ok(0) => { return; }
                Ok(_) => {}
                Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock || e.kind() == std::io::ErrorKind::TimedOut => {
                    // Session read timed out — test might not have sent yet
                    thread::sleep(Duration::from_millis(50));
                    continue;
                }
                Err(_) => { return; }
            }

            for resp in &responses {
                writeln!(session_writer, "{}", serde_json::to_string(resp).unwrap()).ok();
                session_writer.flush().ok();
            }
        }
    })
}

#[test]
fn protocol_handshake() {
    let tmp = tempfile::TempDir::new().unwrap();
    let path = tmp.path().join("agent.sock");
    let path_str = path.to_string_lossy().to_string();
    let done = Arc::new(Mutex::new(false));
    let server = start_mock_server(&path_str, done.clone());

    thread::sleep(Duration::from_millis(300));

    let mut conn = Connection::connect(&path_str).unwrap();
    let resp = conn.handshake(None).unwrap();
    match resp {
        ServerMessage::SessionReady {
            profile,
            model,
            pending_proposals,
        } => {
            assert_eq!(profile, "test-profile");
            assert_eq!(model, "test-model");
            assert_eq!(pending_proposals, vec!["p-test"]);
        }
        other => panic!("expected SessionReady, got: {:?}", other),
    }

    *done.lock().unwrap() = true;
    server.join().unwrap();
}

#[test]
fn protocol_status_request() {
    let tmp = tempfile::TempDir::new().unwrap();
    let path = tmp.path().join("agent2.sock");
    let path_str = path.to_string_lossy().to_string();
    let done = Arc::new(Mutex::new(false));
    let server = start_mock_server(&path_str, done.clone());

    thread::sleep(Duration::from_millis(300));

    let mut conn = Connection::connect(&path_str).unwrap();
    conn.handshake(None).unwrap();

    conn.send(&ClientMessage::Status {
        target: "system".to_string(),
    })
    .unwrap();
    let resp = conn.recv().unwrap();
    match resp {
        ServerMessage::StatusResponse { target, data } => {
            assert_eq!(target, "system");
            assert!(data.get("output").is_some());
        }
        other => panic!("expected StatusResponse, got: {:?}", other),
    }

    *done.lock().unwrap() = true;
    server.join().unwrap();
}

#[test]
fn protocol_audit_request() {
    let tmp = tempfile::TempDir::new().unwrap();
    let path = tmp.path().join("agent3.sock");
    let path_str = path.to_string_lossy().to_string();
    let done = Arc::new(Mutex::new(false));
    let server = start_mock_server(&path_str, done.clone());

    thread::sleep(Duration::from_millis(300));

    let mut conn = Connection::connect(&path_str).unwrap();
    conn.handshake(None).unwrap();

    conn.send(&ClientMessage::Audit {
        action: AuditRequestAction::List,
        query: None,
        id: None,
        limit: 10,
    })
    .unwrap();
    let resp = conn.recv().unwrap();
    match resp {
        ServerMessage::AuditResponse { entries } => {
            assert_eq!(entries.len(), 1);
            assert_eq!(entries[0].get("id").and_then(|v| v.as_str()), Some("a-001"));
        }
        other => panic!("expected AuditResponse, got: {:?}", other),
    }

    *done.lock().unwrap() = true;
    server.join().unwrap();
}

#[test]
fn protocol_chat_turn() {
    let tmp = tempfile::TempDir::new().unwrap();
    let path = tmp.path().join("agent4.sock");
    let path_str = path.to_string_lossy().to_string();
    let done = Arc::new(Mutex::new(false));
    let server = start_mock_server(&path_str, done.clone());

    thread::sleep(Duration::from_millis(300));

    let mut conn = Connection::connect(&path_str).unwrap();
    conn.handshake(None).unwrap();

    conn.send(&ClientMessage::Chat {
        prompt: "inspect system".to_string(),
    })
    .unwrap();

    let msg1 = conn.recv().unwrap();
    match msg1 {
        ServerMessage::ToolCall { name, status, .. } => {
            assert_eq!(name, "inspect");
            assert!(matches!(status, ToolCallStatus::Running));
        }
        other => panic!("expected ToolCall, got: {:?}", other),
    }

    let msg2 = conn.recv().unwrap();
    match msg2 {
        ServerMessage::ToolResult { name, success, .. } => {
            assert_eq!(name, "inspect");
            assert!(success);
        }
        other => panic!("expected ToolResult, got: {:?}", other),
    }

    let msg3 = conn.recv().unwrap();
    match msg3 {
        ServerMessage::TurnComplete { content } => {
            assert!(content.contains("Inspection complete"));
        }
        other => panic!("expected TurnComplete, got: {:?}", other),
    }

    *done.lock().unwrap() = true;
    server.join().unwrap();
}

#[test]
fn connection_reconnect() {
    let tmp = tempfile::TempDir::new().unwrap();
    let path = tmp.path().join("agent5.sock");
    let path_str = path.to_string_lossy().to_string();
    let done = Arc::new(Mutex::new(false));
    let server = start_mock_server(&path_str, done.clone());

    thread::sleep(Duration::from_millis(300));

    let mut conn = Connection::connect(&path_str).unwrap();
    conn.handshake(None).unwrap();

    conn.reconnect().unwrap();

    let resp = conn.handshake(None).unwrap();
    match resp {
        ServerMessage::SessionReady { profile, .. } => {
            assert_eq!(profile, "test-profile");
        }
        other => panic!("expected SessionReady, got: {:?}", other),
    }

    *done.lock().unwrap() = true;
    server.join().unwrap();
}

#[test]
fn connection_rejects_nonexistent_socket() {
    let result = Connection::connect("/tmp/agntos-settings-nonexistent-test.sock");
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("Failed to connect"));
}

#[test]
fn session_turn_lifecycle() {
    let tmp = tempfile::TempDir::new().unwrap();
    let path = tmp.path().join("turn_lifecycle.sock");
    let path_str = path.to_string_lossy().to_string();
    let done = Arc::new(Mutex::new(false));
    let server = serve_mock(&path_str, done.clone());

    thread::sleep(Duration::from_millis(300));

    let mut conn = Connection::connect(&path_str).unwrap();
    conn.handshake(None).unwrap();

    let mut script = UnixStream::connect(&path_str).unwrap();

    let responses = serde_json::json!([
        {"type": "token", "content": "Hello"},
        {"type": "token", "content": " world"},
        {"type": "token", "content": "!"},
        {"type": "turn_complete", "content": "Hello world!"}
    ]);
    writeln!(script, "{}", responses.to_string()).unwrap();

    conn.send(&ClientMessage::Chat {
        prompt: "say hello".to_string(),
    })
    .unwrap();

    let mut session = AppSession::new();

    for _ in 0..4 {
        let msg = conn.recv().unwrap();
        session.handle_server_message(&msg);
    }

    assert_eq!(session.chat.entries.len(), 1);
    assert!(matches!(
        session.chat.entries[0].entry_type,
        agntos_settings::models::ChatEntryType::AssistantText
    ));
    assert_eq!(session.chat.entries[0].content, "Hello world!");
    assert!(matches!(session.turn_state, agntos_settings::session::TurnState::Completed));

    *done.lock().unwrap() = true;
    server.join().unwrap();
}

#[test]
fn session_thinking_filtered() {
    let tmp = tempfile::TempDir::new().unwrap();
    let path = tmp.path().join("thinking_filtered.sock");
    let path_str = path.to_string_lossy().to_string();
    let done = Arc::new(Mutex::new(false));
    let server = serve_mock(&path_str, done.clone());

    thread::sleep(Duration::from_millis(300));

    let mut conn = Connection::connect(&path_str).unwrap();
    conn.handshake(None).unwrap();

    let mut script = UnixStream::connect(&path_str).unwrap();

    let responses = serde_json::json!([
        {"type": "token", "content": "<think>Let me reason"},
        {"type": "token", "content": " step by step"},
        {"type": "token", "content": "</think>"},
        {"type": "token", "content": "The answer is 42"},
        {"type": "turn_complete", "content": "The answer is 42"}
    ]);
    writeln!(script, "{}", responses.to_string()).unwrap();

    conn.send(&ClientMessage::Chat {
        prompt: "think about this".to_string(),
    })
    .unwrap();

    let mut session = AppSession::new();

    for _ in 0..5 {
        let msg = conn.recv().unwrap();
        session.handle_server_message(&msg);
    }

    assert_eq!(session.chat.entries.len(), 1);
    assert!(matches!(
        session.chat.entries[0].entry_type,
        agntos_settings::models::ChatEntryType::AssistantText
    ));
    assert_eq!(session.chat.entries[0].content, "The answer is 42");
    assert!(!session.in_thinking);

    *done.lock().unwrap() = true;
    server.join().unwrap();
}

#[test]
fn session_tool_call_turn() {
    use agntos_settings::models::ChatEntryType;
    use agntos_settings::session::TurnState;

    let tmp = tempfile::TempDir::new().unwrap();
    let path = tmp.path().join("tool_call_turn.sock");
    let path_str = path.to_string_lossy().to_string();
    let done = Arc::new(Mutex::new(false));
    let server = start_mock_server(&path_str, done.clone());

    thread::sleep(Duration::from_millis(300));

    let mut conn = Connection::connect(&path_str).unwrap();
    conn.handshake(None).unwrap();

    conn.send(&ClientMessage::Chat {
        prompt: "inspect system".to_string(),
    })
    .unwrap();

    let mut session = AppSession::new();

    for _ in 0..3 {
        let msg = conn.recv().unwrap();
        session.handle_server_message(&msg);
    }

    assert_eq!(session.chat.entries.len(), 2);
    assert!(matches!(
        session.chat.entries[0].entry_type,
        ChatEntryType::ToolResult
    ));
    assert_eq!(session.chat.entries[0].content, "CPU: 8 cores");
    assert_eq!(session.chat.entries[0].tool_success, Some(true));
    assert!(matches!(session.turn_state, TurnState::Completed));

    *done.lock().unwrap() = true;
    server.join().unwrap();
}

#[test]
fn session_approval_turn() {
    let tmp = tempfile::TempDir::new().unwrap();
    let path = tmp.path().join("approval_turn.sock");
    let path_str = path.to_string_lossy().to_string();
    let done = Arc::new(Mutex::new(false));
    let server = serve_mock(&path_str, done.clone());

    thread::sleep(Duration::from_millis(300));

    let mut conn = Connection::connect(&path_str).unwrap();
    conn.handshake(None).unwrap();

    let mut script = UnixStream::connect(&path_str).unwrap();

    let responses1 = serde_json::json!([
        {"type": "token", "content": "I need approval"},
        {"type": "approval_request", "proposal_id": "p-abc", "summary": "Install nginx", "tool_call_id": "tc-1"}
    ]);
    writeln!(script, "{}", responses1.to_string()).unwrap();

    conn.send(&ClientMessage::Chat {
        prompt: "deploy nginx".to_string(),
    })
    .unwrap();

    let mut session = AppSession::new();

    for _ in 0..2 {
        let msg = conn.recv().unwrap();
        session.handle_server_message(&msg);
    }

    assert_eq!(session.chat.entries.len(), 2);
    assert!(matches!(
        session.chat.entries[0].entry_type,
        agntos_settings::models::ChatEntryType::AssistantText
    ));
    assert_eq!(session.chat.entries[0].content, "I need approval");
    assert!(matches!(
        session.chat.entries[1].entry_type,
        agntos_settings::models::ChatEntryType::ApprovalRequest
    ));
    assert_eq!(
        session.chat.entries[1].proposal_id.as_deref(),
        Some("p-abc")
    );
    assert_eq!(
        session.chat.entries[1].proposal_summary.as_deref(),
        Some("Install nginx")
    );
    assert!(matches!(session.turn_state, agntos_settings::session::TurnState::AwaitingApproval));

    let responses2 = serde_json::json!([
        {"type": "token", "content": " Approved"},
        {"type": "turn_complete", "content": "Done"}
    ]);
    writeln!(script, "{}", responses2.to_string()).unwrap();

    conn.send(&ClientMessage::Approve {
        proposal_id: "p-abc".to_string(),
    })
    .unwrap();

    for _ in 0..2 {
        let msg = conn.recv().unwrap();
        session.handle_server_message(&msg);
    }

    assert_eq!(session.chat.entries.len(), 3);
    assert!(matches!(
        session.chat.entries[2].entry_type,
        agntos_settings::models::ChatEntryType::AssistantText
    ));
    assert_eq!(session.chat.entries[2].content, " Approved");
    assert!(matches!(session.turn_state, agntos_settings::session::TurnState::Completed));

    *done.lock().unwrap() = true;
    server.join().unwrap();
}

#[test]
fn session_event_proposal_created() {
    let tmp = tempfile::TempDir::new().unwrap();
    let path = tmp.path().join("event_proposal.sock");
    let path_str = path.to_string_lossy().to_string();
    let done = Arc::new(Mutex::new(false));
    let server = serve_mock(&path_str, done.clone());

    thread::sleep(Duration::from_millis(300));

    let mut conn = Connection::connect(&path_str).unwrap();
    conn.handshake(None).unwrap();

    let mut script = UnixStream::connect(&path_str).unwrap();

    let responses = serde_json::json!([
        {"type": "event", "event": "proposal_created", "data": {"proposal_id": "p-new", "summary": "Install htop"}}
    ]);
    writeln!(script, "{}", responses.to_string()).unwrap();

    conn.send(&ClientMessage::Status {
        target: "system".to_string(),
    })
    .unwrap();

    let mut session = AppSession::new();

    let msg = conn.recv().unwrap();
    session.handle_server_message(&msg);

    assert_eq!(session.proposals.len(), 1);
    assert_eq!(session.proposals[0].proposal_id, "p-new");
    assert_eq!(session.proposals[0].summary, "Install htop");

    *done.lock().unwrap() = true;
    server.join().unwrap();
}
