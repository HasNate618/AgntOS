#![allow(dead_code)]
mod backend;
mod bridge;
mod models;

use bridge::AppBridge;
use qmetaobject::*;

fn main() {
    let socket_path =
        std::env::var("AGNTOS_SOCKET").unwrap_or_else(|_| "/run/agntd/agent.sock".to_string());

    let mut engine = QmlEngine::new();

    let bridge = AppBridge::new(&socket_path);
    let boxed = QObjectBox::new(bridge);
    let pinned = boxed.pinned();
    {
        let mut b = pinned.borrow_mut();
        b.connect_to_agent();
        b.refresh_proposals();
    }
    engine.set_object_property(QString::from("appBridge"), pinned);

    engine.load_file(QString::from("main.qml"));
    engine.exec();
}
