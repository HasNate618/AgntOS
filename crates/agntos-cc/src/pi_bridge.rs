use agnt_common::models::ModelsConfig;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tauri::{AppHandle, Emitter};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::{Child, ChildStdin, ChildStdout};
use tokio::sync::Mutex;

fn resolve_agntctl(config: &crate::config::AppConfig) -> String {
    let name = &config.agntctl_path;
    let path = Path::new(name);

    if path.is_absolute() {
        if path.exists() {
            return path.to_string_lossy().into_owned();
        }
        return name.clone();
    }

    if let Ok(paths) = std::env::var("PATH") {
        for dir in paths.split(':') {
            let candidate = Path::new(dir).join(name);
            if candidate.exists() {
                return candidate.to_string_lossy().into_owned();
            }
        }
    }

    for dir in &[
        "/usr/local/bin",
        "/mnt/agntos-src/target/release",
        "/home/developer/target/release",
    ] {
        let candidate = Path::new(dir).join(name);
        if candidate.exists() {
            return candidate.to_string_lossy().into_owned();
        }
    }

    name.clone()
}

fn collect_system_context() -> String {
    let generation = std::process::Command::new("nixos-rebuild")
        .args(["list-generations"])
        .output()
        .ok()
        .and_then(|o| {
            String::from_utf8(o.stdout)
                .ok()
                .and_then(|s| s.lines().nth(1).map(|l| l.to_string()))
        })
        .unwrap_or_default();

    let disk = std::process::Command::new("df")
        .args(["-h", "/"])
        .output()
        .ok()
        .and_then(|o| {
            String::from_utf8(o.stdout).ok().and_then(|s| {
                s.lines().nth(1).map(|l| {
                    let parts: Vec<&str> = l.split_whitespace().collect();
                    format!("{} used / {} total", parts.get(2).unwrap_or(&"?"), parts.get(1).unwrap_or(&"?"))
                })
            })
        })
        .unwrap_or_default();

    let memory = std::process::Command::new("free")
        .args(["-h"])
        .output()
        .ok()
        .and_then(|o| {
            String::from_utf8(o.stdout).ok().and_then(|s| {
                s.lines().nth(1).map(|l| {
                    let parts: Vec<&str> = l.split_whitespace().collect();
                    format!("{} used / {} total", parts.get(2).unwrap_or(&"?"), parts.get(1).unwrap_or(&"?"))
                })
            })
        })
        .unwrap_or_default();

    let mut ctx = String::from("--- System Context ---\n");
    if !generation.is_empty() {
        ctx.push_str(&format!("Generation: {}\n", generation));
    }
    if !disk.is_empty() {
        ctx.push_str(&format!("Disk: {}\n", disk));
    }
    if !memory.is_empty() {
        ctx.push_str(&format!("Memory: {}\n", memory));
    }
    ctx.push_str("--- End System Context ---\n\n");
    ctx
}

fn pi_agent_dir() -> PathBuf {
    if let Ok(home) = std::env::var("HOME") {
        PathBuf::from(home).join(".pi/agent")
    } else {
        PathBuf::from("/root/.pi/agent")
    }
}

fn write_pi_models_json(agent_dir: &Path, config: &crate::config::AppConfig) {
    if let Ok(cfg) = ModelsConfig::from_path(&config.model_config_path) {
        let _ = crate::model_catalog::write_pi_models_json(&cfg, agent_dir);
        return;
    }
    if config.llm_base_url.is_empty() {
        return;
    }
    let fallback = serde_json::json!({
        "providers": {
            "local-llama": {
                "baseUrl": config.llm_base_url,
                "api": "openai-completions",
                "apiKey": "not-needed",
                "models": [{
                    "id": "Llama Server",
                    "name": "Llama Server",
                    "reasoning": true,
                    "input": ["text"],
                    "contextWindow": 131072,
                    "maxTokens": 8192,
                    "cost": { "input": 0, "output": 0, "cacheRead": 0, "cacheWrite": 0 }
                }]
            }
        }
    });
    let _ = std::fs::write(
        agent_dir.join("models.json"),
        serde_json::to_string_pretty(&fallback).unwrap_or_default(),
    );
}

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
        // Write Pi models.json with custom provider config
        let agent_dir = pi_agent_dir();
        std::fs::create_dir_all(&agent_dir).ok();
        write_pi_models_json(&agent_dir, &config);

        let mut cmd = tokio::process::Command::new(&config.pi_binary);
        cmd.arg("--mode")
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
            .stderr(std::process::Stdio::piped());

        // Resolve agntctl and pass absolute path to Pi extension
        let agntctl_path = resolve_agntctl(&config);
        cmd.env("AGNTCTL_PATH", &agntctl_path);
        cmd.env(
            "AGNTOS_CONFIG_DIR",
            config.config_dir.to_string_lossy().as_ref(),
        );

        // Also ensure agntctl's directory is on PATH for the subprocess
        let agntctl_dir = Path::new(&agntctl_path).parent().and_then(|p| {
            if p.as_os_str().is_empty() {
                None
            } else {
                Some(p.to_string_lossy().into_owned())
            }
        });
        if let Some(dir) = agntctl_dir {
            let current_path = std::env::var("PATH").unwrap_or_default();
            cmd.env("PATH", format!("{}:{}", dir, current_path));
        }

        let default_model = config.default_model.clone().unwrap_or_else(|| {
            if config.llm_base_url.is_empty() {
                "default".into()
            } else {
                "local-llama/Llama Server".into()
            }
        });
        cmd.arg("--model").arg(&default_model);

        let mut child = cmd.spawn()?;

        let stdin = child.stdin.take().expect("stdin not available");
        let stdout = child.stdout.take().expect("stdout not available");
        let stderr = child.stderr.take().expect("stderr not available");

        // Read stderr in a separate task to prevent pipe deadlock
        tokio::spawn(async move {
            let reader = tokio::io::BufReader::new(stderr);
            let mut lines = reader.lines();
            while let Ok(Some(_)) = lines.next_line().await {
                // discard
            }
        });

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
            let enriched = match command {
                PiCommand::Prompt { message, streaming_behavior } => {
                    let context = collect_system_context();
                    PiCommand::Prompt {
                        message: format!("{}{}", context, message),
                        streaming_behavior: streaming_behavior.clone(),
                    }
                }
                _ => command.clone(),
            };
            let json = serde_json::to_string(&enriched)?;
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
