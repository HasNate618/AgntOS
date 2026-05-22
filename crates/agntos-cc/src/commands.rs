use crate::pi_bridge::{ConnectionStatus, PiBridge, PiCommand};
use std::sync::Arc;
use tauri::State;
use tokio::sync::Mutex;

pub struct BridgeState(pub Arc<Mutex<Option<PiBridge>>>);

#[tauri::command]
pub async fn send_prompt(state: State<'_, BridgeState>, message: String) -> Result<(), String> {
    let guard = state.0.lock().await;
    if let Some(ref bridge) = *guard {
        bridge
            .send_command(&PiCommand::Prompt {
                message,
                streaming_behavior: None,
            })
            .await
            .map_err(|e| e.to_string())
    } else {
        Err("Agent backend not initialized".into())
    }
}

#[tauri::command]
pub async fn send_steer(state: State<'_, BridgeState>, message: String) -> Result<(), String> {
    let guard = state.0.lock().await;
    if let Some(ref bridge) = *guard {
        bridge
            .send_command(&PiCommand::Steer { message })
            .await
            .map_err(|e| e.to_string())
    } else {
        Err("Agent backend not initialized".into())
    }
}

#[tauri::command]
pub async fn send_abort(state: State<'_, BridgeState>) -> Result<(), String> {
    let guard = state.0.lock().await;
    if let Some(ref bridge) = *guard {
        bridge
            .send_command(&PiCommand::Abort)
            .await
            .map_err(|e| e.to_string())
    } else {
        Err("Agent backend not initialized".into())
    }
}

#[tauri::command]
pub async fn set_model(
    state: State<'_, BridgeState>,
    provider: String,
    model_id: String,
) -> Result<(), String> {
    let guard = state.0.lock().await;
    if let Some(ref bridge) = *guard {
        bridge
            .send_command(&PiCommand::SetModel { provider, model_id })
            .await
            .map_err(|e| e.to_string())
    } else {
        Err("Agent backend not initialized".into())
    }
}

#[tauri::command]
pub async fn new_session(state: State<'_, BridgeState>) -> Result<(), String> {
    let guard = state.0.lock().await;
    if let Some(ref bridge) = *guard {
        bridge
            .send_command(&PiCommand::NewSession)
            .await
            .map_err(|e| e.to_string())
    } else {
        Err("Agent backend not initialized".into())
    }
}

#[tauri::command]
pub async fn switch_session(
    state: State<'_, BridgeState>,
    session_path: String,
) -> Result<(), String> {
    let guard = state.0.lock().await;
    if let Some(ref bridge) = *guard {
        bridge
            .send_command(&PiCommand::SwitchSession { session_path })
            .await
            .map_err(|e| e.to_string())
    } else {
        Err("Agent backend not initialized".into())
    }
}

#[tauri::command]
pub async fn send_extension_ui_response(
    state: State<'_, BridgeState>,
    id: String,
    confirmed: bool,
) -> Result<(), String> {
    let guard = state.0.lock().await;
    if let Some(ref bridge) = *guard {
        bridge
            .send_command(&PiCommand::ExtensionUiResponse { id, confirmed })
            .await
            .map_err(|e| e.to_string())
    } else {
        Err("Agent backend not initialized".into())
    }
}

#[tauri::command]
pub async fn get_available_models(
    state: State<'_, BridgeState>,
) -> Result<serde_json::Value, String> {
    let guard = state.0.lock().await;
    if let Some(ref bridge) = *guard {
        // Send get_available_models RPC and read response from events
        bridge
            .send_command(&PiCommand::GetAvailableModels)
            .await
            .map_err(|e| e.to_string())?;
        // Return placeholder - frontend will listen for the response event
        Ok(serde_json::json!({"status": "requested"}))
    } else {
        Err("Agent backend not initialized".into())
    }
}

#[tauri::command]
pub async fn get_connection_status(
    state: State<'_, BridgeState>,
) -> Result<ConnectionStatus, String> {
    let guard = state.0.lock().await;
    if let Some(ref bridge) = *guard {
        Ok(bridge.get_status().await)
    } else {
        Ok(ConnectionStatus {
            connected: false,
            model: None,
            state: "disconnected".into(),
        })
    }
}

use std::path::Path;

#[tauri::command]
pub async fn get_system_info() -> Result<serde_json::Value, String> {
    let agntctl = std::env::var("AGNTCTL_PATH").unwrap_or_else(|_| {
        // Search PATH, then dev paths
        if let Ok(paths) = std::env::var("PATH") {
            for dir in paths.split(':') {
                let candidate = Path::new(dir).join("agntctl");
                if candidate.exists() {
                    return candidate.to_string_lossy().into_owned();
                }
            }
        }
        let dev = Path::new("/mnt/agntos-src/target/release/agntctl");
        if dev.exists() {
            return dev.to_string_lossy().into_owned();
        }
        "agntctl".into()
    });

    let output = std::process::Command::new(&agntctl)
        .args(["inspect", "system"])
        .output()
        .map_err(|e| e.to_string())?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    serde_json::from_str(&stdout).map_err(|e| e.to_string())
}

fn resolve_agntctl() -> String {
    std::env::var("AGNTCTL_PATH").unwrap_or_else(|_| {
        if let Ok(paths) = std::env::var("PATH") {
            for dir in paths.split(':') {
                let candidate = Path::new(dir).join("agntctl");
                if candidate.exists() {
                    return candidate.to_string_lossy().into_owned();
                }
            }
        }
        let dev = Path::new("/mnt/agntos-src/target/release/agntctl");
        if dev.exists() {
            return dev.to_string_lossy().into_owned();
        }
        "agntctl".into()
    })
}

#[tauri::command]
fn agntos_config_dir() -> std::path::PathBuf {
    std::env::var("AGNTOS_CONFIG_DIR")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|_| std::path::PathBuf::from("/etc/agntos"))
}

#[tauri::command]
pub async fn list_proposals() -> Result<serde_json::Value, String> {
    let proposals_dir = agntos_config_dir().join("proposals");
    if !proposals_dir.exists() {
        return Ok(serde_json::json!([]));
    }
    let mut proposals = Vec::new();
    let entries = std::fs::read_dir(proposals_dir).map_err(|e| e.to_string())?;
    for entry in entries {
        let entry = entry.map_err(|e| e.to_string())?;
        let path = entry.path();
        if path.extension().map_or(false, |e| e == "json") {
            let content = std::fs::read_to_string(&path).map_err(|e| e.to_string())?;
            let parsed: serde_json::Value =
                serde_json::from_str(&content).map_err(|e| e.to_string())?;
            let id = parsed
                .get("id")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            let prompt = parsed
                .get("prompt")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());
            let timestamp = entry
                .metadata()
                .ok()
                .and_then(|m| m.modified().ok())
                .and_then(|t| {
                    Some(
                        t.duration_since(std::time::UNIX_EPOCH)
                            .ok()?
                            .as_secs()
                            .to_string(),
                    )
                })
                .unwrap_or_default();
            proposals.push(serde_json::json!({
                "id": id,
                "prompt": prompt,
                "timestamp": timestamp,
                "status": "pending",
            }));
        }
    }
    Ok(serde_json::json!(proposals))
}

#[tauri::command]
pub async fn apply_proposal(id: String) -> Result<String, String> {
    let agntctl = resolve_agntctl();
    let output = std::process::Command::new(&agntctl)
        .args([
            "apply",
            "--config-dir",
            &agntos_config_dir().to_string_lossy(),
            &id,
        ])
        .output()
        .map_err(|e| format!("Failed to execute agntctl: {}", e))?;
    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    } else {
        Err(String::from_utf8_lossy(&output.stderr).to_string())
    }
}

#[tauri::command]
pub async fn list_audit_entries(limit: Option<i32>) -> Result<serde_json::Value, String> {
    let agntctl = resolve_agntctl();
    let limit_str = limit.unwrap_or(20).to_string();
    let output = std::process::Command::new(&agntctl)
        .args(["audit", "list", "--limit", &limit_str, "--json"])
        .output()
        .map_err(|e| format!("Failed to execute agntctl: {}", e))?;
    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        serde_json::from_str(&stdout).map_err(|e| format!("Failed to parse audit output: {}", e))
    } else {
        Err(String::from_utf8_lossy(&output.stderr).to_string())
    }
}

fn pi_sessions_dir() -> std::path::PathBuf {
    if let Ok(home) = std::env::var("HOME") {
        std::path::PathBuf::from(home).join(".pi/agent/sessions")
    } else {
        std::path::PathBuf::from("/root/.pi/agent/sessions")
    }
}

fn config_dir_flag() -> Vec<String> {
    let dir = std::env::var("AGNTOS_CONFIG_DIR").unwrap_or_else(|_| "/etc/agntos".into());
    vec!["--config-dir".into(), dir]
}

#[tauri::command]
pub async fn list_sessions() -> Result<serde_json::Value, String> {
    let dir = pi_sessions_dir();
    if !dir.exists() {
        return Ok(serde_json::json!([]));
    }
    let mut sessions = Vec::new();
    let entries = std::fs::read_dir(&dir).map_err(|e| e.to_string())?;
    for entry in entries {
        let entry = entry.map_err(|e| e.to_string())?;
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("jsonl") {
            continue;
        }
        let meta = entry.metadata().map_err(|e| e.to_string())?;
        let modified = meta
            .modified()
            .ok()
            .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
            .map(|d| d.as_secs())
            .unwrap_or(0);
        let title = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("Session")
            .to_string();
        sessions.push(serde_json::json!({
            "path": path.to_string_lossy(),
            "title": title,
            "modified": modified,
        }));
    }
    sessions.sort_by(|a, b| {
        b.get("modified")
            .and_then(|v| v.as_u64())
            .cmp(&a.get("modified").and_then(|v| v.as_u64()))
    });
    Ok(serde_json::json!(sessions))
}

#[tauri::command]
pub async fn get_models_config() -> Result<serde_json::Value, String> {
    let agntctl = resolve_agntctl();
    let mut args = vec!["model".into(), "list".into(), "--json".into()];
    args.extend(config_dir_flag());
    let output = std::process::Command::new(&agntctl)
        .args(&args)
        .output()
        .map_err(|e| format!("Failed to execute agntctl: {}", e))?;
    if !output.status.success() {
        return Err(String::from_utf8_lossy(&output.stderr).to_string());
    }
    let stdout = String::from_utf8_lossy(&output.stdout);
    serde_json::from_str(&stdout).map_err(|e| format!("Failed to parse models config: {}", e))
}

#[tauri::command]
pub async fn add_model_provider(
    name: String,
    endpoint: String,
    api_key_env: Option<String>,
) -> Result<String, String> {
    let agntctl = resolve_agntctl();
    let mut args = vec![
        "model".into(),
        "add".into(),
        name,
        "--endpoint".into(),
        endpoint,
    ];
    if let Some(env) = api_key_env {
        args.push("--api-key-env".into());
        args.push(env);
    }
    args.extend(config_dir_flag());
    let output = std::process::Command::new(&agntctl)
        .args(&args)
        .output()
        .map_err(|e| format!("Failed to execute agntctl: {}", e))?;
    if !output.status.success() {
        return Err(String::from_utf8_lossy(&output.stderr).to_string());
    }
    let path = crate::model_catalog::models_path();
    if let Ok(cfg) = agnt_common::models::ModelsConfig::from_path(&path) {
        let agent_dir = pi_agent_dir();
        let _ = std::fs::create_dir_all(&agent_dir);
        let _ = crate::model_catalog::write_pi_models_json(&cfg, &agent_dir);
    }
    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

#[tauri::command]
pub async fn list_model_catalog() -> Result<serde_json::Value, String> {
    let path = crate::model_catalog::models_path();
    let cfg = agnt_common::models::ModelsConfig::from_path(&path)?;
    Ok(crate::model_catalog::build_catalog(&cfg))
}

#[tauri::command]
pub async fn probe_provider_models(
    endpoint: String,
    api_key_env: Option<String>,
) -> Result<serde_json::Value, String> {
    let api_key = api_key_env
        .as_ref()
        .and_then(|k| std::env::var(k).ok())
        .unwrap_or_else(|| "not-needed".into());
    let models = crate::model_catalog::fetch_openai_models(&endpoint, &api_key)?;
    let list: Vec<_> = models
        .into_iter()
        .map(|(id, name)| serde_json::json!({ "id": id, "name": name }))
        .collect();
    Ok(serde_json::json!({ "models": list }))
}

#[tauri::command]
pub async fn set_chat_model(provider: String, model_id: String) -> Result<(), String> {
    let path = crate::model_catalog::models_path();
    let mut cfg = agnt_common::models::ModelsConfig::from_path(&path)?;
    if provider == "default" {
        cfg.default.model = model_id.clone();
    } else if let Some(p) = cfg.profiles.get_mut(&provider) {
        p.model = model_id.clone();
    } else {
        return Err(format!("Unknown provider '{}'", provider));
    }
    let toml = cfg.to_toml_string()?;
    std::fs::write(&path, toml).map_err(|e| format!("Failed to write {}: {}", path.display(), e))?;
    let agent_dir = pi_agent_dir();
    let _ = crate::model_catalog::write_pi_models_json(&cfg, &agent_dir);
    Ok(())
}

fn pi_agent_dir() -> std::path::PathBuf {
    if let Ok(home) = std::env::var("HOME") {
        std::path::PathBuf::from(home).join(".pi/agent")
    } else {
        std::path::PathBuf::from("/root/.pi/agent")
    }
}

#[tauri::command]
pub async fn remove_model_profile(name: String) -> Result<String, String> {
    let agntctl = resolve_agntctl();
    let mut args = vec!["model".into(), "remove".into(), name];
    args.extend(config_dir_flag());
    let output = std::process::Command::new(&agntctl)
        .args(&args)
        .output()
        .map_err(|e| format!("Failed to execute agntctl: {}", e))?;
    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    } else {
        Err(String::from_utf8_lossy(&output.stderr).to_string())
    }
}

#[tauri::command]
pub async fn rollback_to(generation: Option<i32>) -> Result<String, String> {
    let agntctl = resolve_agntctl();
    let mut args: Vec<String> = vec!["rollback".into()];
    if let Some(gen) = generation {
        args.push("--generation".into());
        args.push(gen.to_string());
    }
    let output = std::process::Command::new(&agntctl)
        .args(&args)
        .output()
        .map_err(|e| format!("Failed to execute agntctl: {}", e))?;
    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    } else {
        Err(String::from_utf8_lossy(&output.stderr).to_string())
    }
}
