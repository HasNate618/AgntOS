use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tauri::{AppHandle, Emitter};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader, BufWriter};
use tokio::process::{Child, ChildStdin, ChildStdout};
use tokio::sync::Mutex;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum PiCommand {
    #[serde(rename = "prompt")]
    Prompt {
        message: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        streaming_behavior: Option<String>,
    },
    #[serde(rename = "steer")]
    Steer { message: String },
    #[serde(rename = "abort")]
    Abort,
    #[serde(rename = "set_model")]
    SetModel { provider: String, model_id: String },
    #[serde(rename = "new_session")]
    NewSession,
    #[serde(rename = "switch_session")]
    SwitchSession { session_path: String },
    #[serde(rename = "get_state")]
    GetState,
    #[serde(rename = "get_available_models")]
    GetAvailableModels,
    #[serde(rename = "extension_ui_response")]
    ExtensionUiResponse { id: String, confirmed: bool },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionStatus {
    pub connected: bool,
    pub model: Option<String>,
    pub state: String,
}

#[derive(Debug)]
struct PiProcess {
    child: Child,
    stdin: ChildStdin,
}

pub struct PiBridge {
    app_handle: AppHandle,
    process: Arc<Mutex<Option<PiProcess>>>,
    status: Arc<Mutex<ConnectionStatus>>,
}

impl PiBridge {
    pub async fn start(
        config: crate::config::AppConfig,
        app_handle: AppHandle,
    ) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        // Read the AgntOS system prompt to replace Pi's default
        let system_prompt = std::fs::read_to_string(&config.system_prompt_path)
            .unwrap_or_else(|_| "You are the AgntOS system agent.".into());

        let status = Arc::new(Mutex::new(ConnectionStatus {
            connected: false,
            model: None,
            state: "idle".into(),
        }));
        let process = Arc::new(Mutex::new(None));
        let status_clone = status.clone();
        let handle_clone = app_handle.clone();

        // Spawn Pi with identity-stripping flags
        // --no-builtin-tools: disables Pi's read/write/edit/bash, only agntos_* tools exist
        // --no-extensions: prevents auto-discovery of user's ~/.pi/agent/extensions/
        // --no-skills: prevents auto-discovery of user's ~/.pi/agent/skills/ and ~/.agents/skills/
        // --no-context-files: prevents AGENTS.md leakage from user's Pi config
        // -e (--extension): explicitly loads ONLY the agntos-tools extension
        // --system-prompt: replaces Pi's default prompt with pure AgntOS instructions
        let mut child = tokio::process::Command::new(&config.pi_binary)
            .arg("--mode")
            .arg("rpc")
            .arg("--no-builtin-tools")
            .arg("--no-extensions")
            .arg("--no-skills")
            .arg("--no-context-files")
            .arg("-e")
            .arg(&config.extension_path)
            .arg("--system-prompt")
            .arg(&system_prompt)
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()?;

        let stdin = child.stdin.take().expect("stdin not available");
        let stdout = child.stdout.take().expect("stdout not available");

        *process.lock().await = Some(PiProcess { child, stdin });
        *status.lock().await = ConnectionStatus {
            connected: true,
            model: config.default_model.clone(),
            state: "idle".into(),
        };

        app_handle.emit("agent:connected", ())?;

        let proc = process.clone();
        tokio::spawn(async move {
            Self::read_events(stdout, handle_clone, status_clone).await;
            let mut guard = proc.lock().await;
            *guard = None;
        });

        Ok(Self {
            app_handle,
            process,
            status,
        })
    }

    async fn read_events(
        stdout: ChildStdout,
        app_handle: AppHandle,
        status: Arc<Mutex<ConnectionStatus>>,
    ) {
        let reader = BufReader::new(stdout);
        let mut lines = reader.lines();

        while let Ok(Some(line)) = lines.next_line().await {
            if let Ok(event) = serde_json::from_str::<serde_json::Value>(&line) {
                let event_type = event.get("type").and_then(|t| t.as_str()).unwrap_or("");

                match event_type {
                    "message_update" => {
                        let mut s = status.lock().await;
                        s.state = "streaming".into();
                        drop(s);
                        let _ = app_handle.emit("agent:message-update", &line);
                    }
                    "tool_execution_start" => {
                        let _ = app_handle.emit("agent:tool-start", &line);
                    }
                    "tool_execution_end" => {
                        let _ = app_handle.emit("agent:tool-end", &line);
                    }
                    "agent_end" => {
                        let mut s = status.lock().await;
                        s.state = "idle".into();
                        drop(s);
                        let _ = app_handle.emit("agent:end", &line);
                    }
                    "agent_start" => {
                        let mut s = status.lock().await;
                        s.state = "thinking".into();
                        drop(s);
                        let _ = app_handle.emit("agent:start", ());
                    }
                    "extension_ui_request" => {
                        let _ = app_handle.emit("agent:approval-request", &line);
                    }
                    "error" => {
                        let _ = app_handle.emit("agent:error", &line);
                    }
                    "turn_start" | "turn_end" => {}
                    "response" => {
                        // Route RPC responses to frontend for specific commands
                        if let Some(cmd) = event.get("command").and_then(|c| c.as_str()) {
                            match cmd {
                                "get_available_models" | "get_state" | "set_model" => {
                                    let _ = app_handle.emit("agent:rpc-response", &line);
                                }
                                _ => {}
                            }
                        }
                    }
                    _ => {
                        let _ = app_handle.emit("agent:unknown-event", &line);
                    }
                }
            }
        }

        let mut s = status.lock().await;
        s.connected = false;
        s.state = "disconnected".into();
        drop(s);
        let _ = app_handle.emit("agent:disconnected", ());
    }

    pub async fn send_command(
        &self,
        command: &PiCommand,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut guard = self.process.lock().await;
        if let Some(ref mut proc) = *guard {
            let json = serde_json::to_string(command)?;
            proc.stdin.write_all(json.as_bytes()).await?;
            proc.stdin.write_all(b"\n").await?;
            proc.stdin.flush().await?;
            Ok(())
        } else {
            Err("Agent backend not running".into())
        }
    }

    pub async fn get_status(&self) -> ConnectionStatus {
        self.status.lock().await.clone()
    }
}
