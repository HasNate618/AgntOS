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
pub async fn list_proposals() -> Result<serde_json::Value, String> {
    let proposals_dir = Path::new("/etc/agntos/proposals");
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
        .args(["apply", "--config-dir", "/etc/agntos", &id])
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
