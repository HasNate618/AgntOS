#![allow(non_snake_case)]

use crate::backend::Connection;
use agnt_common::wire::*;
use qmetaobject::*;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

fn str_qv(s: &str) -> QVariant {
    QVariant::from(QString::from(s))
}

fn map_insert(m: &mut QVariantMap, k: &str, v: QVariant) {
    m.insert(QString::from(k), v);
}

fn json_to_qvm(val: &serde_json::Value) -> QVariant {
    match val {
        serde_json::Value::String(s) => str_qv(s),
        serde_json::Value::Bool(b) => QVariant::from(*b),
        serde_json::Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                QVariant::from(i as i32)
            } else {
                str_qv(&n.to_string())
            }
        }
        serde_json::Value::Object(obj) => {
            let mut m = QVariantMap::default();
            for (k, v) in obj {
                m.insert(QString::from(k.as_str()), json_to_qvm(v));
            }
            QVariant::from(m)
        }
        serde_json::Value::Array(arr) => {
            let list: QVariantList = arr.iter().map(json_to_qvm).collect();
            QVariant::from(list)
        }
        serde_json::Value::Null => QVariant::from(false),
    }
}

fn make_entry(entry_type: &str, pairs: &[(&str, &str)]) -> serde_json::Value {
    let mut obj = serde_json::Map::new();
    obj.insert(
        "entryType".into(),
        serde_json::Value::String(entry_type.into()),
    );
    for (k, v) in pairs {
        obj.insert(k.to_string(), serde_json::Value::String(v.to_string()));
    }
    serde_json::Value::Object(obj)
}

fn make_tool_call_entry(id: &str, name: &str, args: &str, status: &str) -> serde_json::Value {
    let mut obj = serde_json::Map::new();
    obj.insert(
        "entryType".into(),
        serde_json::Value::String("tool_call".into()),
    );
    obj.insert("toolName".into(), serde_json::Value::String(name.into()));
    obj.insert("toolId".into(), serde_json::Value::String(id.into()));
    obj.insert("toolArgs".into(), serde_json::Value::String(args.into()));
    obj.insert(
        "toolStatus".into(),
        serde_json::Value::String(status.into()),
    );
    obj.insert("content".into(), serde_json::Value::String(String::new()));
    serde_json::Value::Object(obj)
}

fn make_approval_entry(proposal_id: &str, summary: &str) -> serde_json::Value {
    let mut obj = serde_json::Map::new();
    obj.insert(
        "entryType".into(),
        serde_json::Value::String("approval".into()),
    );
    obj.insert(
        "proposalId".into(),
        serde_json::Value::String(proposal_id.into()),
    );
    obj.insert(
        "proposalSummary".into(),
        serde_json::Value::String(summary.into()),
    );
    serde_json::Value::Object(obj)
}

fn entries_to_qvariant(entries: &[serde_json::Value]) -> QVariant {
    let list: QVariantList = entries.iter().map(|e| json_to_qvm(e)).collect();
    QVariant::from(list)
}

fn make_audit_list(entries: &[serde_json::Value]) -> QVariantList {
    entries
        .iter()
        .map(|e| {
            let mut m = QVariantMap::default();
            map_insert(
                &mut m,
                "auditId",
                str_qv(e.get("id").and_then(|v| v.as_str()).unwrap_or("")),
            );
            map_insert(
                &mut m,
                "timestamp",
                str_qv(e.get("timestamp").and_then(|v| v.as_str()).unwrap_or("")),
            );
            map_insert(
                &mut m,
                "actionType",
                str_qv(
                    e.get("action")
                        .and_then(|a| a.get("type"))
                        .and_then(|v| v.as_str())
                        .unwrap_or(""),
                ),
            );
            map_insert(
                &mut m,
                "summary",
                str_qv(e.get("summary").and_then(|v| v.as_str()).unwrap_or("")),
            );
            map_insert(
                &mut m,
                "status",
                str_qv(
                    e.get("result")
                        .and_then(|r| r.get("status"))
                        .and_then(|v| v.as_str())
                        .unwrap_or("unknown"),
                ),
            );
            map_insert(
                &mut m,
                "prompt",
                str_qv(e.get("prompt").and_then(|v| v.as_str()).unwrap_or("")),
            );
            map_insert(
                &mut m,
                "actor",
                str_qv(e.get("actor").and_then(|v| v.as_str()).unwrap_or("")),
            );
            map_insert(
                &mut m,
                "rationale",
                str_qv(e.get("rationale").and_then(|v| v.as_str()).unwrap_or("")),
            );
            map_insert(
                &mut m,
                "rollbackHint",
                str_qv(
                    e.get("rollback_hint")
                        .and_then(|v| v.as_str())
                        .unwrap_or(""),
                ),
            );
            map_insert(
                &mut m,
                "resultMessage",
                str_qv(
                    e.get("result")
                        .and_then(|r| r.get("message"))
                        .and_then(|v| v.as_str())
                        .unwrap_or(""),
                ),
            );
            QVariant::from(m)
        })
        .collect()
}

#[derive(Default, QObject)]
pub struct AppBridge {
    pub base: qt_base_class!(trait QObject),

    pub chat_items: qt_property!(QVariant; NOTIFY chatChanged),
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

    pub proposal_items: qt_property!(QVariant; NOTIFY proposalsChanged),
    pub audit_items: qt_property!(QVariant; NOTIFY auditChanged),

    pub chatChanged: qt_signal!(),
    pub processingChanged: qt_signal!(),
    pub statusChanged: qt_signal!(),
    pub proposalsChanged: qt_signal!(),
    pub auditChanged: qt_signal!(),

    pub socket_path: String,
    pub chat_entries: Arc<Mutex<Vec<serde_json::Value>>>,
    pub pending_updates: Arc<AtomicBool>,

    pub clear_chat: qt_method!(
        pub fn clear_chat(&mut self) {
            self.chat_entries.lock().unwrap().clear();
            self.chat_items = QVariant::from(QVariantList::default());
            self.chatChanged();
        }
    ),

    pub connect_to_agent: qt_method!(
        pub fn connect_to_agent(&mut self) {
            let r = Connection::connect(&self.socket_path);
            if let Ok(mut conn) = r {
                let r2 = conn.handshake(Some("/etc/agntos"));
                if let Ok(ServerMessage::SessionReady { profile, model, .. }) = r2 {
                    self.connected = true;
                    self.profile_name = QString::from(profile.as_str());
                    self.model_name = QString::from(model.as_str());
                    self.statusChanged();
                    return;
                }
            }
            self.connected = false;
            self.statusChanged();
        }
    ),

    pub send_chat: qt_method!(
        pub fn send_chat(&mut self, prompt: QString) {
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

            std::thread::spawn(move || {
                let mut conn = match Connection::connect(&socket_path) {
                    Ok(c) => c,
                    Err(e) => {
                        entries.lock().unwrap().push(make_entry(
                            "assistant",
                            &[("content", &format!("Connection error: {}", e))],
                        ));
                        pending.store(true, Ordering::SeqCst);
                        return;
                    }
                };

                if conn.handshake(None).is_err() {
                    entries.lock().unwrap().push(make_entry(
                        "assistant",
                        &[("content", "Failed to connect to agent.")],
                    ));
                    pending.store(true, Ordering::SeqCst);
                    return;
                }

                let msg = ClientMessage::Chat { prompt: prompt_str };
                if conn.send(&msg).is_err() {
                    entries.lock().unwrap().push(make_entry(
                        "assistant",
                        &[("content", "Failed to send message.")],
                    ));
                    pending.store(true, Ordering::SeqCst);
                    return;
                }

                AppBridge::read_chat_responses_thread(&mut conn, &entries, &pending);
            });
        }
    ),

    pub refresh_proposals: qt_method!(
        pub fn refresh_proposals(&mut self) {
            let mut proposals = Vec::new();
            if let Ok(entries) = std::fs::read_dir("/etc/agntos/proposals") {
                for entry in entries.flatten() {
                    let path = entry.path();
                    let is_json = path.extension().map_or(false, |e| e == "json");
                    if !is_json {
                        continue;
                    }
                    let raw = std::fs::read_to_string(&path);
                    if let Ok(raw) = raw {
                        let v = serde_json::from_str::<serde_json::Value>(&raw);
                        if let Ok(v) = v {
                            let id = v.get("id").and_then(|i| i.as_str()).unwrap_or("?");
                            let summary = v.get("summary").and_then(|s| s.as_str()).unwrap_or("");
                            let mut m = QVariantMap::default();
                            map_insert(&mut m, "proposalId", str_qv(id));
                            map_insert(&mut m, "summary", str_qv(summary));
                            map_insert(&mut m, "status", str_qv("pending"));
                            proposals.push(QVariant::from(m));
                        }
                    }
                }
            }
            self.proposal_items = QVariant::from(proposals.into_iter().collect::<QVariantList>());
            self.proposalsChanged();
        }
    ),

    pub refresh_status: qt_method!(
        pub fn refresh_status(&mut self) {
            let mut conn = Connection::connect(&self.socket_path).ok();
            if let Some(ref mut conn) = conn {
                conn.handshake(None).ok();
                let msg = ClientMessage::Status {
                    target: "system".to_string(),
                };
                conn.send(&msg).ok();
                let resp = conn.recv().ok();
                if let Some(ServerMessage::StatusResponse { data, .. }) = resp {
                    let output = data.get("output").and_then(|v| v.as_str()).unwrap_or("");
                    for line in output.lines() {
                        let lower = line.to_lowercase();
                        if lower.contains("cpu") {
                            self.cpu_info = QString::from(line);
                        } else if lower.contains("ram") || lower.contains("memory") {
                            self.ram_used = QString::from(line);
                        } else if lower.contains("disk") {
                            self.disk_used = QString::from(line);
                        }
                    }
                    self.statusChanged();
                }
            }
        }
    ),

    pub approve_proposal: qt_method!(
        pub fn approve_proposal(&mut self, proposal_id: QString) {
            let mut conn = Connection::connect(&self.socket_path).ok();
            if let Some(ref mut conn) = conn {
                conn.handshake(None).ok();
                let msg = ClientMessage::Approve {
                    proposal_id: proposal_id.to_string(),
                };
                conn.send(&msg).ok();
            }
        }
    ),

    pub dismiss_proposal: qt_method!(
        pub fn dismiss_proposal(&mut self, proposal_id: QString) {
            let mut conn = Connection::connect(&self.socket_path).ok();
            if let Some(ref mut conn) = conn {
                conn.handshake(None).ok();
                let msg = ClientMessage::Dismiss {
                    proposal_id: proposal_id.to_string(),
                    reason: Some("User dismissed from GUI".to_string()),
                };
                conn.send(&msg).ok();
            }
        }
    ),

    pub load_audit: qt_method!(
        pub fn load_audit(&mut self, limit: i32) {
            let mut conn = Connection::connect(&self.socket_path).ok();
            if let Some(ref mut conn) = conn {
                conn.handshake(None).ok();
                let msg = ClientMessage::Audit {
                    action: AuditRequestAction::List,
                    query: None,
                    id: None,
                    limit: limit as u32,
                };
                conn.send(&msg).ok();
                if let Ok(ServerMessage::AuditResponse { entries }) = conn.recv() {
                    self.audit_items = QVariant::from(make_audit_list(&entries));
                    self.auditChanged();
                }
            }
        }
    ),

    pub poll_updates: qt_method!(
        pub fn poll_updates(&mut self) {
            if self.pending_updates.swap(false, Ordering::SeqCst) {
                self.chat_items = entries_to_qvariant(&self.chat_entries.lock().unwrap());
                self.chatChanged();
            }
        }
    ),

    pub search_audit: qt_method!(
        pub fn search_audit(&mut self, query: QString) {
            let mut conn = Connection::connect(&self.socket_path).ok();
            if let Some(ref mut conn) = conn {
                conn.handshake(None).ok();
                let msg = ClientMessage::Audit {
                    action: AuditRequestAction::Search,
                    query: Some(query.to_string()),
                    id: None,
                    limit: 50,
                };
                conn.send(&msg).ok();
                if let Ok(ServerMessage::AuditResponse { entries }) = conn.recv() {
                    self.audit_items = QVariant::from(make_audit_list(&entries));
                    self.auditChanged();
                }
            }
        }
    ),
}

impl AppBridge {
    pub fn new(socket_path: &str) -> Self {
        let mut bridge = AppBridge::default();
        bridge.socket_path = socket_path.to_string();
        bridge.chat_items = QVariant::from(QVariantList::default());
        bridge.proposal_items = QVariant::from(QVariantList::default());
        bridge.audit_items = QVariant::from(QVariantList::default());
        bridge
    }

    /// Background thread: reads responses and writes to shared buffer.
    fn read_chat_responses_thread(
        conn: &mut Connection,
        entries: &Arc<Mutex<Vec<serde_json::Value>>>,
        pending: &Arc<AtomicBool>,
    ) {
        loop {
            let r = conn.recv();
            let mut lock = entries.lock().unwrap();
            match r {
                Ok(ServerMessage::Token { content }) => {
                    let can_append = lock.last().map_or(false, |last| {
                        last.get("entryType").and_then(|v| v.as_str()) == Some("assistant")
                    });
                    if can_append {
                        if let Some(last) = lock.last_mut() {
                            if let Some(obj) = last.as_object_mut() {
                                let existing = obj
                                    .get("content")
                                    .and_then(|v| v.as_str())
                                    .unwrap_or_default();
                                obj.insert(
                                    "content".into(),
                                    serde_json::Value::String(format!("{}{}", existing, content)),
                                );
                            }
                        }
                    } else {
                        lock.push(make_entry("assistant", &[("content", &content)]));
                    }
                    pending.store(true, Ordering::SeqCst);
                }
                Ok(ServerMessage::ToolCall {
                    id, name, args, status,
                }) => {
                    let s = match status {
                        ToolCallStatus::Running => "running",
                        ToolCallStatus::Done => "done",
                    };
                    let args_str = serde_json::to_string(&args).unwrap_or_default();
                    lock.push(make_tool_call_entry(&id, &name, &args_str, s));
                    pending.store(true, Ordering::SeqCst);
                }
                Ok(ServerMessage::ToolResult {
                    output, success, ..
                }) => {
                    if let Some(last) = lock.last_mut() {
                        if let Some(obj) = last.as_object_mut() {
                            obj.insert("entryType".into(),
                                serde_json::Value::String("tool_result".into()));
                            obj.insert("content".into(), serde_json::Value::String(output));
                            obj.insert("toolSuccess".into(), serde_json::Value::Bool(success));
                            obj.insert("toolStatus".into(),
                                serde_json::Value::String("done".into()));
                            pending.store(true, Ordering::SeqCst);
                        }
                    }
                }
                Ok(ServerMessage::ApprovalRequest {
                    proposal_id, summary, ..
                }) => {
                    lock.push(make_approval_entry(&proposal_id, &summary));
                    pending.store(true, Ordering::SeqCst);
                }
                Ok(ServerMessage::TurnComplete { content }) => {
                    if !content.is_empty() && content != "(cancelled)" {
                        lock.push(make_entry("assistant", &[("content", &content)]));
                        pending.store(true, Ordering::SeqCst);
                    }
                    break;
                }
                Ok(ServerMessage::Error { message }) => {
                    lock.push(make_entry("assistant",
                        &[("content", &format!("Error: {}", message))]));
                    pending.store(true, Ordering::SeqCst);
                    break;
                }
                Ok(_) => {}
                Err(_) => break,
            }
        }
    }
}
