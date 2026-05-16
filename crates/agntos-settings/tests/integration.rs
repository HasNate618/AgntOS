use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::{UnixListener, UnixStream};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use agnt_common::wire::{
    ClientMessage, ServerMessage, AuditRequestAction, ToolCallStatus,
};
use agntos_settings::backend::session::Connection;

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
                entries: vec![
                    serde_json::json!({
                        "id": "a-001",
                        "timestamp": "2025-05-16T14:30:00Z",
                        "action": {"type": "Apply", "proposal_id": "p-abc"},
                        "summary": "Applied: Install nginx",
                        "result": {"status": "Success", "message": "Rebuild ok"},
                        "actor": "agent",
                        "prompt": "install nginx",
                    }),
                ],
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
        ServerMessage::SessionReady { profile, model, pending_proposals } => {
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

    conn.send(&ClientMessage::Status { target: "system".to_string() }).unwrap();
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
    }).unwrap();
    let resp = conn.recv().unwrap();
    match resp {
        ServerMessage::AuditResponse { entries } => {
            assert_eq!(entries.len(), 1);
            assert_eq!(
                entries[0].get("id").and_then(|v| v.as_str()),
                Some("a-001")
            );
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

    conn.send(&ClientMessage::Chat { prompt: "inspect system".to_string() }).unwrap();

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
