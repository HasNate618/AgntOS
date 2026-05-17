#![allow(non_snake_case)]

use crate::backend::Connection;
use agnt_common::wire::*;
use qmetaobject::listmodel::SimpleListItem;
use qmetaobject::*;
use std::cell::RefCell;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

fn make_entry(entry_type: &str, pairs: &[(&str, &str)]) -> serde_json::Value {
    let mut obj = serde_json::Map::new();
    obj.insert("entryType".into(), serde_json::Value::String(entry_type.into()));
    for (k, v) in pairs {
        obj.insert(k.to_string(), serde_json::Value::String(v.to_string()));
    }
    serde_json::Value::Object(obj)
}

fn make_tool_call_entry(id: &str, name: &str, args: &str, status: &str) -> serde_json::Value {
    let mut obj = serde_json::Map::new();
    obj.insert("entryType".into(), serde_json::Value::String("tool_call".into()));
    obj.insert("toolName".into(), serde_json::Value::String(name.into()));
    obj.insert("toolId".into(), serde_json::Value::String(id.into()));
    obj.insert("toolArgs".into(), serde_json::Value::String(args.into()));
    obj.insert("toolStatus".into(), serde_json::Value::String(status.into()));
    obj.insert("content".into(), serde_json::Value::String(String::new()));
    serde_json::Value::Object(obj)
}

fn make_approval_entry(proposal_id: &str, summary: &str) -> serde_json::Value {
    let mut obj = serde_json::Map::new();
    obj.insert("entryType".into(), serde_json::Value::String("approval".into()));
    obj.insert("proposalId".into(), serde_json::Value::String(proposal_id.into()));
    obj.insert("proposalSummary".into(), serde_json::Value::String(summary.into()));
    serde_json::Value::Object(obj)
}

fn make_audit_list(entries: &[serde_json::Value]) -> QVariantList {
    entries.iter().map(|e| {
        let mut m = QVariantMap::default();
        let insert = |m: &mut QVariantMap, k: &str, v: &str| { m.insert(QString::from(k), QVariant::from(QString::from(v))); };
        insert(&mut m, "auditId", e.get("id").and_then(|v| v.as_str()).unwrap_or(""));
        insert(&mut m, "timestamp", e.get("timestamp").and_then(|v| v.as_str()).unwrap_or(""));
        insert(&mut m, "actionType", e.get("action").and_then(|a| a.get("type")).and_then(|v| v.as_str()).unwrap_or(""));
        insert(&mut m, "summary", e.get("summary").and_then(|v| v.as_str()).unwrap_or(""));
        insert(&mut m, "status", e.get("result").and_then(|r| r.get("status")).and_then(|v| v.as_str()).unwrap_or("unknown"));
        insert(&mut m, "prompt", e.get("prompt").and_then(|v| v.as_str()).unwrap_or(""));
        insert(&mut m, "actor", e.get("actor").and_then(|v| v.as_str()).unwrap_or(""));
        QVariant::from(m)
    }).collect()
}

// ── ChatEntry: model item with explicit role names ──────────────────────────

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

fn json_to_chat_entry(val: &serde_json::Value) -> ChatEntry {
    ChatEntry {
        entry_type: val.get("entryType").and_then(|v| v.as_str()).unwrap_or("").to_string(),
        content: val.get("content").and_then(|v| v.as_str()).unwrap_or("").to_string(),
        tool_name: val.get("toolName").and_then(|v| v.as_str()).unwrap_or("").to_string(),
        tool_id: val.get("toolId").and_then(|v| v.as_str()).unwrap_or("").to_string(),
        tool_args: val.get("toolArgs").and_then(|v| v.as_str()).unwrap_or("").to_string(),
        tool_status: val.get("toolStatus").and_then(|v| v.as_str()).unwrap_or("").to_string(),
        tool_success: val.get("toolSuccess").and_then(|v| v.as_bool()).unwrap_or(false),
        proposal_id: val.get("proposalId").and_then(|v| v.as_str()).unwrap_or("").to_string(),
        proposal_summary: val.get("proposalSummary").and_then(|v| v.as_str()).unwrap_or("").to_string(),
    }
}

use qmetaobject::listmodel::SimpleListModel;
pub type ChatModel = SimpleListModel<ChatEntry>;

// ── AppBridge ───────────────────────────────────────────────────────────────

#[derive(Default, QObject)]
pub struct AppBridge {
    pub base: qt_base_class!(trait QObject),

    pub is_processing: qt_property!(bool; NOTIFY processingChanged),

    pub connected: qt_property!(bool; NOTIFY statusChanged),
    pub profile_name: qt_property!(QString; NOTIFY statusChanged),
    pub model_name: qt_property!(QString; NOTIFY statusChanged),
    pub endpoint: qt_property!(QString; NOTIFY statusChanged),
    pub cpu_info: qt_property!(QString; NOTIFY statusChanged),
    pub ram_used: qt_property!(QString; NOTIFY statusChanged),
    pub disk_used: qt_property!(QString; NOTIFY statusChanged),
    pub failed_units: qt_property!(i32; NOTIFY statusChanged),
    pub watchdog_interval: qt_property!(i32; NOTIFY statusChanged),
    pub watchdog_disk_threshold: qt_property!(i32; NOTIFY statusChanged),
    pub watchdog_alert_count: qt_property!(i32; NOTIFY statusChanged),
    pub last_check_time: qt_property!(QString; NOTIFY statusChanged),

    pub proposal_items: qt_property!(QVariantList; NOTIFY proposalsChanged),
    pub audit_items: qt_property!(QVariantList; NOTIFY auditChanged),

    pub chat_model: qt_property!(RefCell<ChatModel>; CONST),

    pub processingChanged: qt_signal!(),
    pub statusChanged: qt_signal!(),
    pub proposalsChanged: qt_signal!(),
    pub auditChanged: qt_signal!(),

    pub socket_path: String,
    pub chat_entries: Arc<Mutex<Vec<serde_json::Value>>>,
    pub pending_updates: Arc<AtomicBool>,
    pub chat_connection: Arc<Mutex<Option<Connection>>>,
    pub is_processing_flag: Arc<AtomicBool>,
    pub retry_count: u64,
    pub backoff_until_ms: u64,

    pub clear_chat: qt_method!(
        pub fn clear_chat(&mut self) {
            self.chat_entries.lock().unwrap().clear();
            self.chat_model.borrow_mut().reset_data(Vec::new());
        }
    ),

    pub connect_to_agent: qt_method!(
        pub fn connect_to_agent(&mut self) {
            let r = Connection::connect(&self.socket_path);
            if let Ok(mut conn) = r {
                let r2 = conn.handshake(Some("/etc/agntos"));
                if let Ok(ServerMessage::SessionReady { profile, model, .. }) = r2 {
                    eprintln!("[bridge] connect_to_agent: SUCCESS profile={}", profile);
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
                let mut entries = self.chat_entries.lock().unwrap();
                entries.push(make_entry("user", &[("content", &prompt_str)]));
            }
            self.pending_updates.store(true, Ordering::SeqCst);

            let socket_path = self.socket_path.clone();
            let entries = self.chat_entries.clone();
            let pending = self.pending_updates.clone();
            let is_proc = self.is_processing_flag.clone();
            let chat_conn = self.chat_connection.clone();

            std::thread::spawn(move || {
                let mut conn = match Connection::connect(&socket_path) {
                    Ok(c) => c,
                    Err(e) => {
                        entries.lock().unwrap().push(make_entry(
                            "assistant", &[("content", &format!("Connection error: {}", e))]));
                        pending.store(true, Ordering::SeqCst);
                        is_proc.store(false, Ordering::SeqCst);
                        return;
                    }
                };
                if conn.handshake(None).is_err() {
                    entries.lock().unwrap().push(make_entry(
                        "assistant", &[("content", "Failed to connect to agent.")]));
                    pending.store(true, Ordering::SeqCst);
                    is_proc.store(false, Ordering::SeqCst);
                    return;
                }
                let msg = ClientMessage::Chat { prompt: prompt_str };
                if conn.send(&msg).is_err() {
                    entries.lock().unwrap().push(make_entry(
                        "assistant", &[("content", "Failed to send message.")]));
                    pending.store(true, Ordering::SeqCst);
                    is_proc.store(false, Ordering::SeqCst);
                    return;
                }

                if let Ok(reader_conn) = conn.try_clone() {
                    *chat_conn.lock().unwrap() = Some(conn);
                    let mut rc = reader_conn;
                    AppBridge::read_chat_responses_thread(&mut rc, &entries, &pending, &is_proc);
                } else {
                    AppBridge::read_chat_responses_thread(&mut conn, &entries, &pending, &is_proc);
                }
            });
        }
    ),

    pub refresh_proposals: qt_method!(
        pub fn refresh_proposals(&mut self) {
            let mut proposals = Vec::new();
            if let Ok(entries) = std::fs::read_dir("/etc/agntos/proposals") {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.extension().map_or(false, |e| e == "json") {
                        if let Ok(raw) = std::fs::read_to_string(&path) {
                            if let Ok(v) = serde_json::from_str::<serde_json::Value>(&raw) {
                                let id = v.get("id").and_then(|i| i.as_str()).unwrap_or("?");
                                let summary = v.get("summary").and_then(|s| s.as_str()).unwrap_or("");
                                let mut m = QVariantMap::default();
                                m.insert(QString::from("proposalId"), QVariant::from(QString::from(id)));
                                m.insert(QString::from("summary"), QVariant::from(QString::from(summary)));
                                m.insert(QString::from("status"), QVariant::from(QString::from("pending")));
                                proposals.push(QVariant::from(m));
                            }
                        }
                    }
                }
            }
            self.proposal_items = proposals.into_iter().collect::<QVariantList>();
            self.proposalsChanged();
        }
    ),

    pub refresh_status: qt_method!(
        pub fn refresh_status(&mut self) {
            let mut conn = match Connection::connect(&self.socket_path) {
                Ok(c) => c,
                Err(_) => return,
            };
            conn.handshake(None).ok();
            let msg = ClientMessage::Status { target: "system".to_string() };
            conn.send(&msg).ok();
            if let Ok(ServerMessage::StatusResponse { data, .. }) = conn.recv() {
                let output = data.get("output").and_then(|v| v.as_str()).unwrap_or("");
                for line in output.lines() {
                    let lower = line.to_lowercase();
                    if lower.contains("cpu") { self.cpu_info = QString::from(line); }
                    else if lower.contains("ram") || lower.contains("memory") { self.ram_used = QString::from(line); }
                    else if lower.contains("disk") { self.disk_used = QString::from(line); }
                }
                self.statusChanged();
            }
        }
    ),

    pub approve_proposal: qt_method!(
        pub fn approve_proposal(&mut self, proposal_id: QString) {
            if let Some(conn) = self.chat_connection.lock().unwrap().as_mut() {
                conn.send(&ClientMessage::Approve { proposal_id: proposal_id.to_string() }).ok();
            } else {
                let mut conn = match Connection::connect(&self.socket_path) {
                    Ok(c) => c,
                    Err(_) => return,
                };
                conn.handshake(None).ok();
                conn.send(&ClientMessage::Approve { proposal_id: proposal_id.to_string() }).ok();
            }
        }
    ),

    pub dismiss_proposal: qt_method!(
        pub fn dismiss_proposal(&mut self, proposal_id: QString) {
            if let Some(conn) = self.chat_connection.lock().unwrap().as_mut() {
                conn.send(&ClientMessage::Dismiss {
                    proposal_id: proposal_id.to_string(),
                    reason: Some("User dismissed from GUI".to_string()),
                }).ok();
            } else {
                let mut conn = match Connection::connect(&self.socket_path) {
                    Ok(c) => c,
                    Err(_) => return,
                };
                conn.handshake(None).ok();
                conn.send(&ClientMessage::Dismiss {
                    proposal_id: proposal_id.to_string(),
                    reason: Some("User dismissed from GUI".to_string()),
                }).ok();
            }
        }
    ),

    pub load_audit: qt_method!(
        pub fn load_audit(&mut self, limit: i32) {
            let mut conn = match Connection::connect(&self.socket_path) {
                Ok(c) => c,
                Err(_) => return,
            };
            conn.handshake(None).ok();
            conn.send(&ClientMessage::Audit {
                action: AuditRequestAction::List, query: None, id: None, limit: limit as u32,
            }).ok();
            if let Ok(ServerMessage::AuditResponse { entries }) = conn.recv() {
                self.audit_items = make_audit_list(&entries);
                self.auditChanged();
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
                if now >= self.backoff_until_ms { self.connect_to_agent(); }
            }

            let proc_val = self.is_processing_flag.load(Ordering::SeqCst);
            if proc_val != self.is_processing {
                self.is_processing = proc_val;
                if !proc_val {
                    *self.chat_connection.lock().unwrap() = None;
                }
                self.processingChanged();
            }

            if self.pending_updates.swap(false, Ordering::SeqCst) {
                let entries = self.chat_entries.lock().unwrap();
                eprintln!("[bridge] poll_updates: {} entries", entries.len());
                for (i, e) in entries.iter().enumerate() {
                    let et = e.get("entryType").and_then(|v| v.as_str()).unwrap_or("?");
                    let c = e.get("content").and_then(|v| v.as_str()).unwrap_or("").chars().take(60).collect::<String>();
                    eprintln!("[bridge]   [{}] type={} c={}", i, et, c);
                }
                let chat_entries: Vec<ChatEntry> = entries.iter().map(json_to_chat_entry).collect();
                self.chat_model.borrow_mut().reset_data(chat_entries);
            }
        }
    ),

    pub search_audit: qt_method!(
        pub fn search_audit(&mut self, query: QString) {
            let mut conn = match Connection::connect(&self.socket_path) {
                Ok(c) => c,
                Err(_) => return,
            };
            conn.handshake(None).ok();
            conn.send(&ClientMessage::Audit {
                action: AuditRequestAction::Search, query: Some(query.to_string()), id: None, limit: 50,
            }).ok();
            if let Ok(ServerMessage::AuditResponse { entries }) = conn.recv() {
                self.audit_items = make_audit_list(&entries);
                self.auditChanged();
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

    fn read_chat_responses_thread(
        conn: &mut Connection,
        entries: &Arc<Mutex<Vec<serde_json::Value>>>,
        pending: &Arc<AtomicBool>,
        is_proc: &Arc<AtomicBool>,
    ) {
        eprintln!("[bridge] read_chat_responses_thread started");
        let mut had_tokens = false;
        loop {
            let r = conn.recv();
            let mut lock = entries.lock().unwrap();
            match r {
                Ok(ServerMessage::Token { content }) => {
                    had_tokens = true;
                    let can_append = lock.last().map_or(false, |last| {
                        last.get("entryType").and_then(|v| v.as_str()) == Some("assistant")
                    });
                    if can_append {
                        if let Some(last) = lock.last_mut() {
                            if let Some(obj) = last.as_object_mut() {
                                let existing = obj.get("content").and_then(|v| v.as_str()).unwrap_or_default();
                                obj.insert("content".into(), serde_json::Value::String(format!("{}{}", existing, content)));
                            }
                        }
                    } else {
                        lock.push(make_entry("assistant", &[("content", &content)]));
                    }
                    pending.store(true, Ordering::SeqCst);
                }
                Ok(ServerMessage::ToolCall { id, name, args, status }) => {
                    let s = match status { ToolCallStatus::Running => "running", ToolCallStatus::Done => "done" };
                    let args_str = serde_json::to_string(&args).unwrap_or_default();
                    eprintln!("[bridge] ToolCall: id={}, name={}", id, name);
                    lock.push(make_tool_call_entry(&id, &name, &args_str, s));
                    pending.store(true, Ordering::SeqCst);
                }
                Ok(ServerMessage::ToolResult { output, success, .. }) => {
                    eprintln!("[bridge] ToolResult: success={}", success);
                    if let Some(last) = lock.last_mut() {
                        if let Some(obj) = last.as_object_mut() {
                            obj.insert("entryType".into(), serde_json::Value::String("tool_result".into()));
                            obj.insert("content".into(), serde_json::Value::String(output));
                            obj.insert("toolSuccess".into(), serde_json::Value::Bool(success));
                            obj.insert("toolStatus".into(), serde_json::Value::String("done".into()));
                            pending.store(true, Ordering::SeqCst);
                        }
                    }
                }
                Ok(ServerMessage::ApprovalRequest { proposal_id, summary, .. }) => {
                    eprintln!("[bridge] ApprovalRequest: id={}, summary={}", proposal_id, summary);
                    lock.push(make_approval_entry(&proposal_id, &summary));
                    pending.store(true, Ordering::SeqCst);
                }
                Ok(ServerMessage::TurnComplete { content }) => {
                    eprintln!("[bridge] TurnComplete: len={}", content.len());
                    if !content.is_empty() && content != "(cancelled)" && !had_tokens {
                        lock.push(make_entry("assistant", &[("content", &content)]));
                        pending.store(true, Ordering::SeqCst);
                    } else if !content.is_empty() && content != "(cancelled)" && had_tokens {
                        // Content already built via streamed tokens; just signal update
                        pending.store(true, Ordering::SeqCst);
                    }
                    drop(lock);
                    is_proc.store(false, Ordering::SeqCst);
                    break;
                }
                Ok(ServerMessage::Error { message }) => {
                    eprintln!("[bridge] Error: {}", message);
                    lock.push(make_entry("assistant", &[("content", &format!("Error: {}", message))]));
                    pending.store(true, Ordering::SeqCst);
                    drop(lock);
                    is_proc.store(false, Ordering::SeqCst);
                    break;
                }
                Ok(_) => {}
                Err(_) => {
                    eprintln!("[bridge] Connection closed");
                    drop(lock);
                    is_proc.store(false, Ordering::SeqCst);
                    break;
                }
            }
        }
        eprintln!("[bridge] read_chat_responses_thread finished");
    }
}
