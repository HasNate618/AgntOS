#![allow(non_snake_case)]

use crate::backend::Connection;
use agnt_common::wire::*;
use qmetaobject::*;

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

    pub chatItems: qt_property!(QVariant; NOTIFY chatChanged),
    pub isProcessing: qt_property!(bool; NOTIFY processingChanged),

    pub connected: qt_property!(bool; NOTIFY statusChanged),
    pub profileName: qt_property!(QString; NOTIFY statusChanged),
    pub modelName: qt_property!(QString; NOTIFY statusChanged),
    pub endpoint: qt_property!(QString; NOTIFY statusChanged),
    pub cpuInfo: qt_property!(QString; NOTIFY statusChanged),
    pub ramUsed: qt_property!(QString; NOTIFY statusChanged),
    pub diskUsed: qt_property!(QString; NOTIFY statusChanged),
    pub failedUnits: qt_property!(i32; NOTIFY statusChanged),
    pub watchdogInterval: qt_property!(i32; NOTIFY statusChanged),
    pub watchdogDiskThreshold: qt_property!(i32; NOTIFY statusChanged),
    pub watchdogAlertCount: qt_property!(i32; NOTIFY statusChanged),
    pub lastCheckTime: qt_property!(QString; NOTIFY statusChanged),

    pub proposalItems: qt_property!(QVariant; NOTIFY proposalsChanged),
    pub auditItems: qt_property!(QVariant; NOTIFY auditChanged),

    pub chatChanged: qt_signal!(),
    pub processingChanged: qt_signal!(),
    pub statusChanged: qt_signal!(),
    pub proposalsChanged: qt_signal!(),
    pub auditChanged: qt_signal!(),

    pub socket_path: String,
    pub chat_entries: Vec<serde_json::Value>,

    pub clear_chat: qt_method!(
        pub fn clear_chat(&mut self) {
            self.chat_entries.clear();
            self.chatItems = QVariant::from(QVariantList::default());
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
                    self.profileName = QString::from(profile.as_str());
                    self.modelName = QString::from(model.as_str());
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
            self.isProcessing = true;
            self.processingChanged();

            let prompt_str = prompt.to_string();
            self.chat_entries
                .push(make_entry("user", &[("content", &prompt_str)]));
            self.sync_chat();

            let mut conn = match Connection::connect(&self.socket_path) {
                Ok(c) => c,
                Err(e) => {
                    self.chat_entries.push(make_entry(
                        "assistant",
                        &[("content", &format!("Connection error: {}", e))],
                    ));
                    self.sync_chat();
                    self.isProcessing = false;
                    self.processingChanged();
                    return;
                }
            };

            if conn.handshake(None).is_err() {
                self.chat_entries.push(make_entry(
                    "assistant",
                    &[("content", "Failed to connect to agent.")],
                ));
                self.sync_chat();
                self.isProcessing = false;
                self.processingChanged();
                return;
            }

            let msg = ClientMessage::Chat { prompt: prompt_str };
            if conn.send(&msg).is_err() {
                self.chat_entries.push(make_entry(
                    "assistant",
                    &[("content", "Failed to send message.")],
                ));
                self.sync_chat();
                self.isProcessing = false;
                self.processingChanged();
                return;
            }

            self.read_chat_responses(&mut conn);

            self.isProcessing = false;
            self.processingChanged();
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
            self.proposalItems = QVariant::from(proposals.into_iter().collect::<QVariantList>());
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
                            self.cpuInfo = QString::from(line);
                        } else if lower.contains("ram") || lower.contains("memory") {
                            self.ramUsed = QString::from(line);
                        } else if lower.contains("disk") {
                            self.diskUsed = QString::from(line);
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
                    self.auditItems = QVariant::from(make_audit_list(&entries));
                    self.auditChanged();
                }
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
                    self.auditItems = QVariant::from(make_audit_list(&entries));
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
        bridge.chatItems = QVariant::from(QVariantList::default());
        bridge.proposalItems = QVariant::from(QVariantList::default());
        bridge.auditItems = QVariant::from(QVariantList::default());
        bridge
    }

    fn sync_chat(&mut self) {
        self.chatItems = entries_to_qvariant(&self.chat_entries);
        self.chatChanged();
    }

    fn read_chat_responses(&mut self, conn: &mut Connection) {
        loop {
            let r = conn.recv();
            match r {
                Ok(ServerMessage::Token { content }) => {
                    let can_append = self.chat_entries.last().map_or(false, |last| {
                        last.get("entryType").and_then(|v| v.as_str()) == Some("assistant")
                    });
                    if can_append {
                        if let Some(last) = self.chat_entries.last_mut() {
                            if let Some(obj) = last.as_object_mut() {
                                let existing = obj
                                    .get("content")
                                    .and_then(|v| v.as_str())
                                    .unwrap_or_default();
                                obj.insert(
                                    "content".into(),
                                    serde_json::Value::String(format!("{}{}", existing, content)),
                                );
                                self.sync_chat();
                            }
                        }
                    } else {
                        self.chat_entries
                            .push(make_entry("assistant", &[("content", &content)]));
                        self.sync_chat();
                    }
                }
                Ok(ServerMessage::ToolCall {
                    id,
                    name,
                    args,
                    status,
                }) => {
                    let s = match status {
                        ToolCallStatus::Running => "running",
                        ToolCallStatus::Done => "done",
                    };
                    let args_str = serde_json::to_string(&args).unwrap_or_default();
                    self.chat_entries
                        .push(make_tool_call_entry(&id, &name, &args_str, s));
                    self.sync_chat();
                }
                Ok(ServerMessage::ToolResult {
                    output, success, ..
                }) => {
                    if let Some(last) = self.chat_entries.last_mut() {
                        if let Some(obj) = last.as_object_mut() {
                            obj.insert(
                                "entryType".into(),
                                serde_json::Value::String("tool_result".into()),
                            );
                            obj.insert("content".into(), serde_json::Value::String(output));
                            obj.insert("toolSuccess".into(), serde_json::Value::Bool(success));
                            obj.insert(
                                "toolStatus".into(),
                                serde_json::Value::String("done".into()),
                            );
                            self.sync_chat();
                        }
                    }
                }
                Ok(ServerMessage::ApprovalRequest {
                    proposal_id,
                    summary,
                    ..
                }) => {
                    self.chat_entries
                        .push(make_approval_entry(&proposal_id, &summary));
                    self.sync_chat();
                }
                Ok(ServerMessage::TurnComplete { content }) => {
                    if !content.is_empty() && content != "(cancelled)" {
                        self.chat_entries
                            .push(make_entry("assistant", &[("content", &content)]));
                        self.sync_chat();
                    }
                    break;
                }
                Ok(ServerMessage::Error { message }) => {
                    self.chat_entries.push(make_entry(
                        "assistant",
                        &[("content", &format!("Error: {}", message))],
                    ));
                    self.sync_chat();
                    break;
                }
                Ok(_) => {}
                Err(_) => break,
            }
        }
    }
}
