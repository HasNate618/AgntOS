use crate::backend::protocol::deserialize;
use agnt_common::wire::{ClientMessage, ServerMessage};
use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::UnixStream;
use std::time::Duration;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)]
pub enum SessionState {
    Disconnected,
    InitSent,
    Ready,
    Chatting,
    AwaitingApproval,
}

#[derive(Debug)]
pub struct Connection {
    stream: UnixStream,
    reader: BufReader<UnixStream>,
    socket_path: String,
    backoff_secs: u64,
}

impl Connection {
    pub fn connect(socket_path: &str) -> Result<Self, String> {
        let stream = UnixStream::connect(socket_path)
            .map_err(|e| format!("Failed to connect to {}: {}", socket_path, e))?;
        let reader = BufReader::new(stream.try_clone().map_err(|e| e.to_string())?);
        Ok(Self {
            stream,
            reader,
            socket_path: socket_path.to_string(),
            backoff_secs: 1,
        })
    }

    pub fn send(&mut self, msg: &ClientMessage) -> Result<(), String> {
        let json = serde_json::to_string(msg).map_err(|e| format!("Serialize error: {}", e))?;
        writeln!(self.stream, "{}", json).map_err(|e| format!("Write error: {}", e))?;
        Ok(())
    }

    pub fn recv(&mut self) -> Result<ServerMessage, String> {
        let mut line = String::new();
        self.reader
            .read_line(&mut line)
            .map_err(|e| format!("Read error: {}", e))?;
        if line.trim().is_empty() {
            return Err("Connection closed by server".to_string());
        }
        deserialize(line.trim())
    }

    pub fn reconnect(&mut self) -> Result<(), String> {
        loop {
            match UnixStream::connect(&self.socket_path) {
                Ok(stream) => {
                    self.stream = stream;
                    self.reader =
                        BufReader::new(self.stream.try_clone().map_err(|e| e.to_string())?);
                    self.backoff_secs = 1;
                    return Ok(());
                }
                Err(_) => {
                    std::thread::sleep(Duration::from_secs(self.backoff_secs));
                    self.backoff_secs = (self.backoff_secs * 2).min(30);
                }
            }
        }
    }

    pub fn handshake(&mut self, config_dir: Option<&str>) -> Result<ServerMessage, String> {
        let init = ClientMessage::Init {
            config_dir: config_dir.map(|s| s.to_string()),
        };
        self.send(&init)?;
        self.recv()
    }

    pub fn socket_path(&self) -> &str {
        &self.socket_path
    }
}

pub struct Session {
    pub state: SessionState,
    pub profile: String,
    pub model: String,
    pub pending_proposals: Vec<String>,
}

impl Session {
    pub fn new() -> Self {
        Self {
            state: SessionState::Disconnected,
            profile: String::new(),
            model: String::new(),
            pending_proposals: Vec::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::os::unix::net::UnixListener;
    use std::sync::{Arc, Mutex};
    use std::thread;

    #[test]
    fn connection_rejects_missing_socket() {
        let result = Connection::connect("/tmp/agntos-settings-test-nonexistent.sock");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Failed to connect"));
    }

    #[test]
    fn session_starts_disconnected() {
        let s = Session::new();
        assert_eq!(s.state, SessionState::Disconnected);
    }

    #[test]
    fn backoff_resets_on_success() {
        let path = "/tmp/agntos-settings-test-backoff.sock";
        let _ = std::fs::remove_file(path);
        let listener = UnixListener::bind(path).unwrap();
        let _ = thread::spawn(move || listener.accept().ok());

        let conn = Connection::connect(path).unwrap();
        assert_eq!(conn.backoff_secs, 1);
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn send_and_recv_roundtrip() {
        let path = "/tmp/agntos-settings-test-roundtrip.sock";
        let _ = std::fs::remove_file(path);
        let listener = UnixListener::bind(path).unwrap();

        let server_done = Arc::new(Mutex::new(false));
        let server_done_clone = server_done.clone();

        let _ = thread::spawn(move || {
            if let Ok((stream, _)) = listener.accept() {
                let mut reader = BufReader::new(&stream);
                let mut line = String::new();
                reader.read_line(&mut line).ok();
                let resp = r#"{"type":"session_ready","profile":"test","model":"test","pending_proposals":[]}"#;
                let mut writer = stream;
                writeln!(writer, "{}", resp).ok();
            }
            *server_done_clone.lock().unwrap() = true;
        });

        std::thread::sleep(std::time::Duration::from_millis(200));
        let mut conn = Connection::connect(path).unwrap();

        let msg = ClientMessage::Init { config_dir: None };
        conn.send(&msg).ok();

        let resp = conn.recv().unwrap();
        match resp {
            ServerMessage::SessionReady { profile, .. } => {
                assert_eq!(profile, "test");
            }
            _ => panic!("expected SessionReady"),
        }

        let _ = std::fs::remove_file(path);
    }
}
