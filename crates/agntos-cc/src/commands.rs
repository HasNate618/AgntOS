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

#[tauri::command]
pub async fn get_system_info() -> Result<serde_json::Value, String> {
    let output = std::process::Command::new("agntctl")
        .args(["inspect", "system"])
        .output()
        .map_err(|e| e.to_string())?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    serde_json::from_str(&stdout).map_err(|e| e.to_string())
}
