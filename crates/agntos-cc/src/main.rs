use agntos_cc::commands::BridgeState;
use agntos_cc::config::AppConfig;
use agntos_cc::pi_bridge::PiBridge;
use std::sync::Arc;
use tauri::Manager;
use tokio::sync::Mutex;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let config = AppConfig::load().unwrap_or_default();

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .manage(BridgeState(Arc::new(Mutex::new(None))))
        .setup(move |app| {
            let handle = app.handle().clone();
            let state = app.state::<BridgeState>();
            let bridge_state = state.0.clone();
            let cfg = config.clone();

            tauri::async_runtime::spawn(async move {
                match PiBridge::start(cfg, handle).await {
                    Ok(bridge) => {
                        *bridge_state.lock().await = Some(bridge);
                    }
                    Err(e) => {
                        tracing::error!("Failed to start agent backend: {e}");
                    }
                }
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            agntos_cc::commands::send_prompt,
            agntos_cc::commands::send_steer,
            agntos_cc::commands::send_abort,
            agntos_cc::commands::set_model,
            agntos_cc::commands::new_session,
            agntos_cc::commands::switch_session,
            agntos_cc::commands::send_extension_ui_response,
            agntos_cc::commands::get_connection_status,
            agntos_cc::commands::get_system_info,
        ])
        .run(tauri::generate_context!())
        .expect("error while running AgntOS Control Centre");
}

fn main() {
    run();
}
