// agntd - AgntOS agent daemon.
//
// Hermes-like local agent with OS integration.
// Handles chat sessions, tool execution, model routing,
// skill loading, memory, and approval flows.

use std::sync::Arc;
use tokio::sync::Mutex;

mod session;
mod tools;
mod routing;

pub struct AgntDaemon {
    pub config_dir: String,
    pub sessions: Arc<Mutex<Vec<session::Session>>>,
}

impl AgntDaemon {
    pub fn new(config_dir: &str) -> Self {
        Self {
            config_dir: config_dir.to_string(),
            sessions: Arc::new(Mutex::new(Vec::new())),
        }
    }
}

#[tokio::main]
async fn main() {
    let daemon = AgntDaemon::new("/etc/agntos");

    println!("agntd: AgntOS agent daemon starting...");
    println!("agntd: config dir: {}", daemon.config_dir);
    println!("agntd: ready");

    // In Phase 1, this will:
    // - Load skills from /etc/agntos/skills/
    // - Load model routing config
    // - Open a local socket or CLI interface
    // - Accept chat messages and route to tool execution
    // - Log all actions to the audit log

    // Placeholder: keep running
    loop {
        tokio::time::sleep(tokio::time::Duration::from_secs(3600)).await;
    }
}
