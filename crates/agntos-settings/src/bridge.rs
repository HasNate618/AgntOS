#![allow(non_snake_case)]

use crate::backend::Connection;
use crate::models::proposal_model::{Proposal, ProposalStatus};
use crate::session::{AppSession, AuditEntry, ConnectionState, TurnState};
use agnt_common::wire::*;
use qmetaobject::listmodel::SimpleListItem;
use qmetaobject::*;
use std::cell::RefCell;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex};

// ── ChatEntry: model item for QML ──────────────────────────────────────────

#[derive(Clone, Default)]
pub struct ChatEntry {
    pub entry_type: String,
    pub content: String,
    pub tool_name: String,
    pub tool_id: String,
    pub tool_args: String,
    pub tool_status: String,
    pub tool_success: bool,
    pub proposal_id: String,
    pub proposal_summary: String,
}

fn to_qml_chat_entry(e: &crate::models::chat_model::ChatEntry) -> ChatEntry {
    ChatEntry {
        entry_type: format!("{:?}", e.entry_type).to_lowercase(),
        content: e.content.clone(),
        tool_name: e.tool_name.as_deref().unwrap_or("").to_string(),
        tool_id: e.tool_id.as_deref().unwrap_or("").to_string(),
        tool_args: e
            .tool_args
            .as_ref()
            .map(|v| v.to_string())
            .unwrap_or_default(),
        tool_status: e.tool_status.as_deref().unwrap_or("").to_string(),
        tool_success: e.tool_success.unwrap_or(false),
        proposal_id: e.proposal_id.as_deref().unwrap_or("").to_string(),
        proposal_summary: e.proposal_summary.as_deref().unwrap_or("").to_string(),
    }
}

impl SimpleListItem for ChatEntry {
    fn get(&self, role: i32) -> QVariant {
        match role {
            0 => QVariant::from(QString::from(self.entry_type.as_str())),
            1 => QVariant::from(QString::from(self.content.as_str())),
            2 => QVariant::from(QString::from(self.tool_name.as_str())),
            3 => QVariant::from(QString::from(self.tool_id.as_str())),
            4 => QVariant::from(QString::from(self.tool_args.as_str())),
            5 => QVariant::from(QString::from(self.tool_status.as_str())),
            6 => QVariant::from(self.tool_success),
            7 => QVariant::from(QString::from(self.proposal_id.as_str())),
            8 => QVariant::from(QString::from(self.proposal_summary.as_str())),
            _ => QVariant::default(),
        }
    }
    fn names() -> Vec<QByteArray> {
        vec![
            QByteArray::from("entryType"),
            QByteArray::from("content"),
            QByteArray::from("toolName"),
            QByteArray::from("toolId"),
            QByteArray::from("toolArgs"),
            QByteArray::from("toolStatus"),
            QByteArray::from("toolSuccess"),
            QByteArray::from("proposalId"),
            QByteArray::from("proposalSummary"),
        ]
    }
}

use qmetaobject::listmodel::SimpleListModel;
pub type ChatModel = SimpleListModel<ChatEntry>;

fn make_proposal_list(proposals: &[Proposal]) -> QVariantList {
    proposals
        .iter()
        .map(|p| {
            let mut m = QVariantMap::default();
            m.insert(
                QString::from("proposalId"),
                QVariant::from(QString::from(p.proposal_id.as_str())),
            );
            m.insert(
                QString::from("summary"),
                QVariant::from(QString::from(p.summary.as_str())),
            );
            m.insert(
                QString::from("nixChanges"),
                QVariant::from(QString::from(p.nix_changes.as_str())),
            );
            m.insert(
                QString::from("rollbackGuidance"),
                QVariant::from(QString::from(p.rollback_guidance.as_str())),
            );
            let status = match &p.status {
                ProposalStatus::Pending => "pending",
                ProposalStatus::Applied => "applied",
                ProposalStatus::Dismissed => "dismissed",
            };
            m.insert(
                QString::from("status"),
                QVariant::from(QString::from(status)),
            );
            m.insert(
                QString::from("createdAt"),
                QVariant::from(QString::from(p.created_at.as_str())),
            );
            QVariant::from(m)
        })
        .collect()
}

fn make_audit_list(entries: &[AuditEntry]) -> QVariantList {
    entries
        .iter()
        .map(|e| {
            let mut m = QVariantMap::default();
            m.insert(
                QString::from("auditId"),
                QVariant::from(QString::from(e.audit_id.as_str())),
            );
            m.insert(
                QString::from("timestamp"),
                QVariant::from(QString::from(e.timestamp.as_str())),
            );
            m.insert(
                QString::from("actionType"),
                QVariant::from(QString::from(e.action_type.as_str())),
            );
            m.insert(
                QString::from("summary"),
                QVariant::from(QString::from(e.summary.as_str())),
            );
            m.insert(
                QString::from("status"),
                QVariant::from(QString::from(e.status.as_str())),
            );
            m.insert(
                QString::from("prompt"),
                QVariant::from(QString::from(e.prompt.as_str())),
            );
            m.insert(
                QString::from("actor"),
                QVariant::from(QString::from(e.actor.as_str())),
            );
            QVariant::from(m)
        })
        .collect()
}

// ── AppBridge ───────────────────────────────────────────────────────────────

#[derive(Default, QObject)]
pub struct AppBridge {
    pub base: qt_base_class!(trait QObject),

    pub is_processing: qt_property!(bool; NOTIFY processingChanged),
    pub turn_state: qt_property!(QString; NOTIFY stateChanged),
    pub connection_state: qt_property!(QString; NOTIFY stateChanged),

    pub connected: qt_property!(bool; NOTIFY statusChanged),
    pub profile_name: qt_property!(QString; NOTIFY statusChanged),
    pub model_name: qt_property!(QString; NOTIFY statusChanged),
    pub cpu_info: qt_property!(QString; NOTIFY statusChanged),
    pub ram_used: qt_property!(QString; NOTIFY statusChanged),
    pub disk_used: qt_property!(QString; NOTIFY statusChanged),
    pub failed_units: qt_property!(i32; NOTIFY statusChanged),
    pub watchdog_alert_count: qt_property!(i32; NOTIFY statusChanged),
    pub last_check_time: qt_property!(QString; NOTIFY statusChanged),

    pub proposal_items: qt_property!(QVariantList; NOTIFY proposalsChanged),
    pub audit_items: qt_property!(QVariantList; NOTIFY auditChanged),

    pub chat_model: qt_property!(RefCell<ChatModel>; CONST),

    pub processingChanged: qt_signal!(),
    pub stateChanged: qt_signal!(),
    pub statusChanged: qt_signal!(),
    pub proposalsChanged: qt_signal!(),
    pub auditChanged: qt_signal!(),

    pub socket_path: String,
    pub session: Arc<Mutex<AppSession>>,
    pub session_change: Arc<AtomicU64>,
    pub last_seen_change: u64,
    pub chat_connection: Arc<Mutex<Option<Connection>>>,
    pub is_processing_flag: Arc<AtomicBool>,
    pub retry_count: u64,
    pub backoff_until_ms: u64,

    pub clear_chat: qt_method!(
        pub fn clear_chat(&mut self) {
            let mut s = self.session.lock().unwrap();
            s.chat.clear();
            self.session_change.fetch_add(1, Ordering::SeqCst);
        }
    ),

    pub connect_to_agent: qt_method!(
        pub fn connect_to_agent(&mut self) {
            let r = Connection::connect(&self.socket_path);
            if let Ok(mut conn) = r {
                let r2 = conn.handshake(Some("/etc/agntos"));
                if let Ok(ServerMessage::SessionReady { profile, model, .. }) = r2 {
                    eprintln!("[bridge] connect_to_agent: SUCCESS profile={}", profile);
                    let mut s = self.session.lock().unwrap();
                    s.connection_state = ConnectionState::Connected;
                    s.profile = profile.clone();
                    s.model = model.clone();
                    self.session_change.fetch_add(1, Ordering::SeqCst);
                    self.connected = true;
                    self.retry_count = 0;
                    self.backoff_until_ms = 0;
                    self.profile_name = QString::from(profile.as_str());
                    self.model_name = QString::from(model.as_str());
                    self.statusChanged();
                    return;
                }
                eprintln!("[bridge] connect_to_agent: handshake failed");
            } else {
                eprintln!("[bridge] connect_to_agent: connect failed");
            }
            self.connected = false;
            self.retry_count += 1;
            let delay = (100u64 << self.retry_count.min(8)).min(30000);
            self.backoff_until_ms = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_millis() as u64 + delay)
                .unwrap_or(0);
            eprintln!("[bridge] connect_to_agent: FAILED (retry in {}ms)", delay);
            self.statusChanged();
        }
    ),

    pub send_chat: qt_method!(
        pub fn send_chat(&mut self, prompt: QString) {
            eprintln!("[bridge] send_chat called: len={}", prompt.len());
            self.is_processing_flag.store(true, Ordering::SeqCst);
            self.is_processing = true;
            self.processingChanged();

            let prompt_str = prompt.to_string();
            {
                let mut s = self.session.lock().unwrap();
                s.chat.add_user_message(&prompt_str);
            }
            self.session_change.fetch_add(1, Ordering::SeqCst);

            let socket_path = self.socket_path.clone();
            let session = self.session.clone();
            let change_cnt = self.session_change.clone();
            let is_proc = self.is_processing_flag.clone();
            let chat_conn = self.chat_connection.clone();

            std::thread::spawn(move || {
                let mut conn = match Connection::connect(&socket_path) {
                    Ok(c) => c,
                    Err(e) => {
                        let mut s = session.lock().unwrap();
                        s.chat
                            .add_assistant_text(&format!("Connection error: {}", e));
                        s.turn_state = TurnState::Error {
                            message: e.to_string(),
                        };
                        change_cnt.fetch_add(1, Ordering::SeqCst);
                        is_proc.store(false, Ordering::SeqCst);
                        return;
                    }
                };
                if conn.handshake(None).is_err() {
                    let mut s = session.lock().unwrap();
                    s.chat.add_assistant_text("Failed to connect to agent.");
                    s.turn_state = TurnState::Error {
                        message: "handshake failed".to_string(),
                    };
                    change_cnt.fetch_add(1, Ordering::SeqCst);
                    is_proc.store(false, Ordering::SeqCst);
                    return;
                }
                let msg = ClientMessage::Chat { prompt: prompt_str };
                if conn.send(&msg).is_err() {
                    let mut s = session.lock().unwrap();
                    s.chat.add_assistant_text("Failed to send message.");
                    s.turn_state = TurnState::Error {
                        message: "send failed".to_string(),
                    };
                    change_cnt.fetch_add(1, Ordering::SeqCst);
                    is_proc.store(false, Ordering::SeqCst);
                    return;
                }

                if let Ok(reader_conn) = conn.try_clone() {
                    *chat_conn.lock().unwrap() = Some(conn);
                    let mut rc = reader_conn;
                    AppBridge::read_chat_responses_thread(&mut rc, &session, &change_cnt, &is_proc);
                } else {
                    AppBridge::read_chat_responses_thread(
                        &mut conn,
                        &session,
                        &change_cnt,
                        &is_proc,
                    );
                }
            });
        }
    ),

    pub refresh_proposals: qt_method!(
        pub fn refresh_proposals(&mut self) {
            if let Some(conn) = self.chat_connection.lock().unwrap().as_mut() {
                conn.send(&ClientMessage::Status {
                    target: "proposals".to_string(),
                })
                .ok();
                return;
            }
            // Fallback: try to load from session if connected
            let s = self.session.lock().unwrap();
            self.proposal_items = make_proposal_list(&s.proposals);
            self.proposalsChanged();
        }
    ),

    pub refresh_status: qt_method!(
        pub fn refresh_status(&mut self) {
            if let Some(conn) = self.chat_connection.lock().unwrap().as_mut() {
                conn.send(&ClientMessage::Status {
                    target: "system".to_string(),
                })
                .ok();
            } else {
                let mut conn = match Connection::connect(&self.socket_path) {
                    Ok(c) => c,
                    Err(_) => return,
                };
                conn.handshake(None).ok();
                conn.send(&ClientMessage::Status {
                    target: "system".to_string(),
                })
                .ok();
            }
        }
    ),

    pub approve_proposal: qt_method!(
        pub fn approve_proposal(&mut self, proposal_id: QString) {
            if let Some(conn) = self.chat_connection.lock().unwrap().as_mut() {
                conn.send(&ClientMessage::Approve {
                    proposal_id: proposal_id.to_string(),
                })
                .ok();
            } else {
                let mut conn = match Connection::connect(&self.socket_path) {
                    Ok(c) => c,
                    Err(_) => return,
                };
                conn.handshake(None).ok();
                conn.send(&ClientMessage::Approve {
                    proposal_id: proposal_id.to_string(),
                })
                .ok();
            }
        }
    ),

    pub dismiss_proposal: qt_method!(
        pub fn dismiss_proposal(&mut self, proposal_id: QString) {
            if let Some(conn) = self.chat_connection.lock().unwrap().as_mut() {
                conn.send(&ClientMessage::Dismiss {
                    proposal_id: proposal_id.to_string(),
                    reason: Some("User dismissed from GUI".to_string()),
                })
                .ok();
            } else {
                let mut conn = match Connection::connect(&self.socket_path) {
                    Ok(c) => c,
                    Err(_) => return,
                };
                conn.handshake(None).ok();
                conn.send(&ClientMessage::Dismiss {
                    proposal_id: proposal_id.to_string(),
                    reason: Some("User dismissed from GUI".to_string()),
                })
                .ok();
            }
        }
    ),

    pub load_audit: qt_method!(
        pub fn load_audit(&mut self, limit: i32) {
            if let Some(conn) = self.chat_connection.lock().unwrap().as_mut() {
                conn.send(&ClientMessage::Audit {
                    action: AuditRequestAction::List,
                    query: None,
                    id: None,
                    limit: limit as u32,
                })
                .ok();
            } else {
                let mut conn = match Connection::connect(&self.socket_path) {
                    Ok(c) => c,
                    Err(_) => return,
                };
                conn.handshake(None).ok();
                conn.send(&ClientMessage::Audit {
                    action: AuditRequestAction::List,
                    query: None,
                    id: None,
                    limit: limit as u32,
                })
                .ok();
            }
        }
    ),

    pub poll_updates: qt_method!(
        pub fn poll_updates(&mut self) {
            if !self.connected {
                let now = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .map(|d| d.as_millis() as u64)
                    .unwrap_or(0);
                if now >= self.backoff_until_ms {
                    self.connect_to_agent();
                }
            }

            let proc_val = self.is_processing_flag.load(Ordering::SeqCst);
            if proc_val != self.is_processing {
                self.is_processing = proc_val;
                if !proc_val {
                    *self.chat_connection.lock().unwrap() = None;
                }
                self.processingChanged();
            }

            let current_change = self.session_change.load(Ordering::SeqCst);
            if current_change != self.last_seen_change {
                self.last_seen_change = current_change;
                self.sync_from_session();
            }
        }
    ),

    pub search_audit: qt_method!(
        pub fn search_audit(&mut self, query: QString) {
            if let Some(conn) = self.chat_connection.lock().unwrap().as_mut() {
                conn.send(&ClientMessage::Audit {
                    action: AuditRequestAction::Search,
                    query: Some(query.to_string()),
                    id: None,
                    limit: 50,
                })
                .ok();
            } else {
                let mut conn = match Connection::connect(&self.socket_path) {
                    Ok(c) => c,
                    Err(_) => return,
                };
                conn.handshake(None).ok();
                conn.send(&ClientMessage::Audit {
                    action: AuditRequestAction::Search,
                    query: Some(query.to_string()),
                    id: None,
                    limit: 50,
                })
                .ok();
            }
        }
    ),
}

impl AppBridge {
    pub fn new(socket_path: &str) -> Self {
        let mut bridge = AppBridge::default();
        bridge.socket_path = socket_path.to_string();
        bridge.proposal_items = QVariantList::default();
        bridge.audit_items = QVariantList::default();
        bridge
    }

    fn sync_from_session(&mut self) {
        let s = self.session.lock().unwrap();

        self.turn_state = QString::from(s.turn_state.to_string().as_str());

        let conn_state = match s.connection_state {
            ConnectionState::Connected => "connected",
            ConnectionState::Connecting => "connecting",
            ConnectionState::Disconnected => "disconnected",
        };
        self.connection_state = QString::from(conn_state);
        self.connected = matches!(s.connection_state, ConnectionState::Connected);
        self.profile_name = QString::from(s.profile.as_str());
        self.model_name = QString::from(s.model.as_str());

        self.cpu_info = QString::from(s.status.cpu_info.as_str());
        self.ram_used = QString::from(s.status.ram_used.as_str());
        self.disk_used = QString::from(s.status.disk_used.as_str());
        self.failed_units = s.status.failed_units as i32;
        self.watchdog_alert_count = s.status.watchdog_alert_count as i32;
        self.last_check_time = QString::from(s.status.last_check_time.as_str());

        let chat_entries: Vec<ChatEntry> = s.chat.entries.iter().map(to_qml_chat_entry).collect();
        self.chat_model.borrow_mut().reset_data(chat_entries);

        self.proposal_items = make_proposal_list(&s.proposals);
        self.audit_items = make_audit_list(&s.audit);

        drop(s);
        self.stateChanged();
        self.statusChanged();
        self.proposalsChanged();
        self.auditChanged();
    }

    fn read_chat_responses_thread(
        conn: &mut Connection,
        session: &Arc<Mutex<AppSession>>,
        change_cnt: &Arc<AtomicU64>,
        is_proc: &Arc<AtomicBool>,
    ) {
        eprintln!("[bridge] read_chat_responses_thread started");
        loop {
            let r = conn.recv();
            match r {
                Ok(msg) => {
                    let mut s = session.lock().unwrap();
                    s.handle_server_message(&msg);
                    change_cnt.fetch_add(1, Ordering::SeqCst);
                    if matches!(
                        msg,
                        ServerMessage::TurnComplete { .. } | ServerMessage::Error { .. }
                    ) {
                        drop(s);
                        is_proc.store(false, Ordering::SeqCst);
                        break;
                    }
                }
                Err(_) => {
                    eprintln!("[bridge] Connection closed");
                    is_proc.store(false, Ordering::SeqCst);
                    break;
                }
            }
        }
        eprintln!("[bridge] read_chat_responses_thread finished");
    }
}
