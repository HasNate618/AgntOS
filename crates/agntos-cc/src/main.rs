use agntos_cc::commands::BridgeState;
use agntos_cc::config::AppConfig;
use agntos_cc::pi_bridge::PiBridge;
use std::sync::Arc;
use tauri::Manager;
use tokio::sync::Mutex;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()),
        )
        .init();

    let config = AppConfig::load().unwrap_or_default();
    std::env::set_var(
        "AGNTOS_CONFIG_DIR",
        config.config_dir.to_string_lossy().as_ref(),
    );
    tracing::info!(
        "Config: dir={}, pi={}, prompt={}, extension={}",
        config.config_dir.display(),
        config.pi_binary,
        config.system_prompt_path.display(),
        config.extension_path.display()
    );

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .manage(BridgeState(Arc::new(Mutex::new(None))))
        .setup(move |app| {
            let handle = app.handle().clone();
            let state = app.state::<BridgeState>();
            let bridge_state = state.0.clone();
            let cfg = config.clone();

            tauri::async_runtime::spawn(async move {
                tracing::info!("Starting Pi bridge...");
                let bridge_result = PiBridge::start(cfg, handle.clone()).await;
                let connected = bridge_result.is_ok();
                let err = match &bridge_result {
                    Err(e) => e
                        .to_string()
                        .replace('\\', "\\\\")
                        .replace('\'', "\\'"),
                    Ok(_) => String::new(),
                };
                match bridge_result {
                    Ok(bridge) => {
                        *bridge_state.lock().await = Some(bridge);
                        tracing::info!("Pi bridge started successfully");
                    }
                    Err(_) => tracing::error!("Failed to start Pi bridge: {}", err),
                }

                tokio::time::sleep(std::time::Duration::from_millis(500)).await;
                let js = format!(
                    "window.__AGNTOS_BRIDGE_STATUS__ = {{ connected: {}, state: '{}', error: '{}' }}",
                    if connected { "true" } else { "false" },
                    if connected { "idle" } else { "disconnected" },
                    err
                );
                for w in handle.webview_windows().values() {
                    let _ = w.eval(&js);
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
            agntos_cc::commands::get_available_models,
            agntos_cc::commands::get_system_info,
            agntos_cc::commands::list_proposals,
            agntos_cc::commands::apply_proposal,
            agntos_cc::commands::list_audit_entries,
            agntos_cc::commands::rollback_to,
            agntos_cc::commands::list_sessions,
            agntos_cc::commands::get_models_config,
            agntos_cc::commands::add_model_provider,
            agntos_cc::commands::remove_model_profile,
            agntos_cc::commands::list_model_catalog,
            agntos_cc::commands::probe_provider_models,
            agntos_cc::commands::set_chat_model,
        ])
        .run(tauri::generate_context!())
        .expect("error while running AgntOS Control Centre");
}

fn main() {
    run();
}
