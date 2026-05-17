#![allow(dead_code)]
mod backend;
mod bridge;
mod models;
mod session;

use bridge::AppBridge;
use qmetaobject::*;

fn main() {
    let mut engine = QmlEngine::new();

    let socket_path =
        std::env::var("AGNTOS_SOCKET")
            .or_else(|_| std::env::var("XDG_RUNTIME_DIR").map(|d| format!("{}/agntd.sock", d)))
            .unwrap_or_else(|_| "/run/agntd/agent.sock".to_string());

    let qml_dir = std::env::var("AGNTOS_QML_DIR")
        .unwrap_or_else(|_| "/run/current-system/sw/share/agntos-settings/qml".to_string());
    let qml_path = format!("{}/main.qml", qml_dir);

    let bridge = AppBridge::new(&socket_path);
    let boxed = QObjectBox::new(bridge);
    let pinned = boxed.pinned();

    // Blocking startup retry: try to connect before showing the window
    // so the UI starts in "connected" state when agntd is available.
    // Retries every 500ms for up to 30 seconds.
    {
        let mut b = pinned.borrow_mut();
        for _ in 0..60 {
            b.connect_to_agent();
            if b.connected {
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(500));
        }
        b.refresh_proposals();
    }

    engine.set_object_property(QString::from("appBridge"), pinned);

    engine.load_file(QString::from(qml_path.as_str()));
    engine.exec();
}
