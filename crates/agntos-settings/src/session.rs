use crate::models::chat_model::{ChatEntryType, ChatModel};
use crate::models::proposal_model::{Proposal, ProposalStatus};
use agnt_common::wire::*;

#[derive(Debug, Clone, PartialEq)]
pub enum ConnectionState {
    Disconnected,
    Connecting,
    Connected,
}

#[derive(Debug, Clone)]
pub enum TurnState {
    Idle,
    Thinking,
    ToolRunning { name: String, detail: String },
    AwaitingApproval,
    Streaming,
    Completed,
    Error { message: String },
}

impl std::fmt::Display for TurnState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TurnState::Idle => write!(f, "idle"),
            TurnState::Thinking => write!(f, "thinking"),
            TurnState::ToolRunning { name, detail } => {
                write!(f, "tool_running:{}:{}", name, detail)
            }
            TurnState::AwaitingApproval => write!(f, "awaiting_approval"),
            TurnState::Streaming => write!(f, "streaming"),
            TurnState::Completed => write!(f, "completed"),
            TurnState::Error { message } => write!(f, "error:{}", message),
        }
    }
}

#[derive(Debug, Clone)]
pub struct StatusData {
    pub cpu_info: String,
    pub ram_used: String,
    pub disk_used: String,
    pub failed_units: i64,
    pub watchdog_interval: i64,
    pub watchdog_disk_threshold: i64,
    pub watchdog_alert_count: i64,
    pub last_check_time: String,
}

impl Default for StatusData {
    fn default() -> Self {
        Self {
            cpu_info: String::new(),
            ram_used: String::new(),
            disk_used: String::new(),
            failed_units: 0,
            watchdog_interval: 300,
            watchdog_disk_threshold: 95,
            watchdog_alert_count: 0,
            last_check_time: String::new(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct AuditEntry {
    pub audit_id: String,
    pub timestamp: String,
    pub action_type: String,
    pub summary: String,
    pub status: String,
    pub prompt: String,
    pub actor: String,
}

#[derive(Debug, Clone)]
pub struct AppSession {
    pub connection_state: ConnectionState,
    pub turn_state: TurnState,
    pub profile: String,
    pub model: String,
    pub chat: ChatModel,
    pub proposals: Vec<Proposal>,
    pub status: StatusData,
    pub audit: Vec<AuditEntry>,
    pub in_thinking: bool,
}

const THINK_OPEN: &[u8] = &[b'<', b't', b'h', b'i', b'n', b'k', b'>'];
const THINK_CLOSE: &[u8] = &[b'<', b'/', b't', b'h', b'i', b'n', b'k', b'>'];

fn strip_thinking(content: &str, in_thinking: &mut bool) -> Option<String> {
    let mut result = String::new();
    let mut pos = 0;
    let bytes = content.as_bytes();

    while pos < content.len() {
        if *in_thinking {
            if bytes[pos..].starts_with(THINK_CLOSE) {
                pos += THINK_CLOSE.len();
                *in_thinking = false;
            } else {
                pos += 1;
            }
        } else if bytes[pos..].starts_with(THINK_OPEN) {
            *in_thinking = true;
            pos += THINK_OPEN.len();
        } else {
            result.push(bytes[pos] as char);
            pos += 1;
        }
    }

    if result.trim().is_empty() { None } else { Some(result) }
}

impl AppSession {
    pub fn new() -> Self {
        Self {
            connection_state: ConnectionState::Disconnected,
            turn_state: TurnState::Idle,
            profile: String::new(),
            model: String::new(),
            chat: ChatModel::new(),
            proposals: Vec::new(),
            status: StatusData::default(),
            audit: Vec::new(),
            in_thinking: false,
        }
    }

    pub fn handle_server_message(&mut self, msg: &ServerMessage) {
        match msg {
            ServerMessage::SessionReady {
                profile,
                model,
                pending_proposals,
            } => {
                self.connection_state = ConnectionState::Connected;
                self.profile = profile.clone();
                self.model = model.clone();
                for pid in pending_proposals {
                    if !self.proposals.iter().any(|p| p.proposal_id == *pid) {
                        self.proposals.push(Proposal {
                            proposal_id: pid.clone(),
                            summary: String::new(),
                            nix_changes: String::new(),
                            rollback_guidance: String::new(),
                            created_at: String::new(),
                            status: ProposalStatus::Pending,
                        });
                    }
                }
            }

            ServerMessage::Token { content } => {
                let was_thinking = self.in_thinking;
                let filtered = strip_thinking(content, &mut self.in_thinking);

                let entered_thinking = !was_thinking && self.in_thinking;
                let _exited_thinking = was_thinking && !self.in_thinking;

                if entered_thinking && filtered.is_none() {
                    self.turn_state = TurnState::Thinking;
                    return;
                }
                if was_thinking && self.in_thinking {
                    return;
                }

                if let Some(ref c) = filtered {
                    if c.trim().is_empty() {
                        if !entered_thinking {
                            self.turn_state = TurnState::Thinking;
                        }
                        return;
                    }
                    self.turn_state = TurnState::Streaming;
                    let last_is_assistant = self.chat.entries.last().map_or(false, |e| {
                        matches!(e.entry_type, ChatEntryType::AssistantText)
                    });
                    let last_is_tool_result = self.chat.entries.last().map_or(false, |e| {
                        matches!(e.entry_type, ChatEntryType::ToolResult)
                    });
                    let needs_new_text = self.chat.entries.is_empty()
                        || last_is_tool_result
                        || !last_is_assistant;
                    if needs_new_text {
                        self.chat.add_assistant_text(c);
                    } else {
                        self.chat.append_token(c);
                    }
                    if entered_thinking {
                        self.turn_state = TurnState::Thinking;
                    }
                }
            }

            ServerMessage::ToolCall { id, name, args, status } => {
                self.chat.add_tool_call(id, name, args.clone());
                match status {
                    ToolCallStatus::Running => {
                        self.turn_state = TurnState::ToolRunning {
                            name: name.clone(),
                            detail: args.to_string(),
                        };
                    }
                    ToolCallStatus::Done => {
                        if let Some(entry) = self.chat.entries.iter_mut().rev().find(|e| e.tool_id.as_deref() == Some(id.as_str())) {
                            entry.entry_type = ChatEntryType::ToolResult;
                        }
                    }
                }
            }

            ServerMessage::ToolResult { id, output, success, .. } => {
                self.chat.resolve_tool_call(id, output, *success);
                self.turn_state = TurnState::Streaming;
            }

            ServerMessage::ApprovalRequest { proposal_id, summary, tool_call_id } => {
                self.chat.add_approval_request(proposal_id, summary, tool_call_id);
                self.turn_state = TurnState::AwaitingApproval;
            }

            ServerMessage::TurnComplete { content } => {
                let has_last_assistant = self.chat.entries.last().map_or(false, |e| {
                    matches!(e.entry_type, ChatEntryType::AssistantText)
                });
                if !content.is_empty() && content != "(cancelled)" && !has_last_assistant {
                    self.chat.add_assistant_text(content);
                }
                self.turn_state = TurnState::Completed;
            }

            ServerMessage::StatusResponse { data, .. } => {
                let output = data.get("output").and_then(|v| v.as_str()).unwrap_or("");
                for line in output.lines() {
                    let lower = line.to_lowercase();
                    if lower.contains("cpu") {
                        self.status.cpu_info = line.to_string();
                    } else if lower.contains("ram") || lower.contains("memory") {
                        self.status.ram_used = line.to_string();
                    } else if lower.contains("disk") {
                        self.status.disk_used = line.to_string();
                    }
                }
            }

            ServerMessage::AuditResponse { entries } => {
                self.audit = entries
                    .iter()
                    .map(|e| AuditEntry {
                        audit_id: e.get("id").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                        timestamp: e.get("timestamp").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                        action_type: e.get("action")
                            .and_then(|a| a.get("type"))
                            .and_then(|v| v.as_str())
                            .unwrap_or("")
                            .to_string(),
                        summary: e.get("summary").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                        status: e.get("result")
                            .and_then(|r| r.get("status"))
                            .and_then(|v| v.as_str())
                            .unwrap_or("unknown")
                            .to_string(),
                        prompt: e.get("prompt").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                        actor: e.get("actor").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                    })
                    .collect();
            }

            ServerMessage::Event { event, data } => {
                match event.as_str() {
                    "proposal_created" => {
                        if let Some(pid) = data.get("proposal_id").and_then(|v| v.as_str()) {
                            let summary = data.get("summary").and_then(|v| v.as_str()).unwrap_or("");
                            if !self.proposals.iter().any(|p| p.proposal_id == pid) {
                                self.proposals.push(Proposal {
                                    proposal_id: pid.to_string(),
                                    summary: summary.to_string(),
                                    nix_changes: String::new(),
                                    rollback_guidance: String::new(),
                                    created_at: String::new(),
                                    status: ProposalStatus::Pending,
                                });
                            }
                        }
                    }
                    "watchdog_alert" => {
                        self.status.watchdog_alert_count += 1;
                    }
                    "rebuild_started" => {
                        if let Some(pid) = data.get("proposal_id").and_then(|v| v.as_str()) {
                            if let Some(p) = self.proposals.iter_mut().find(|p| p.proposal_id == pid) {
                                p.status = ProposalStatus::Applied;
                            }
                        }
                    }
                    "rebuild_complete" => {
                        if let Some(pid) = data.get("proposal_id").and_then(|v| v.as_str()) {
                            if let Some(p) = self.proposals.iter_mut().find(|p| p.proposal_id == pid) {
                                let success = data.get("success").and_then(|v| v.as_bool()).unwrap_or(false);
                                p.status = if success {
                                    ProposalStatus::Applied
                                } else {
                                    ProposalStatus::Pending
                                };
                            }
                        }
                    }
                    "audit_entry" => {
                        let entry = AuditEntry {
                            audit_id: data.get("id").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                            timestamp: data.get("timestamp").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                            action_type: data.get("action")
                                .and_then(|a| a.get("type"))
                                .and_then(|v| v.as_str())
                                .unwrap_or("")
                                .to_string(),
                            summary: data.get("summary").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                            status: data.get("result")
                                .and_then(|r| r.get("status"))
                                .and_then(|v| v.as_str())
                                .unwrap_or("unknown")
                                .to_string(),
                            prompt: data.get("prompt").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                            actor: data.get("actor").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                        };
                        self.audit.insert(0, entry);
                    }
                    _ => {}
                }
            }

            ServerMessage::Error { message } => {
                self.turn_state = TurnState::Error {
                    message: message.clone(),
                };
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn session_starts_disconnected() {
        let s = AppSession::new();
        assert_eq!(s.connection_state, ConnectionState::Disconnected);
        assert!(matches!(s.turn_state, TurnState::Idle));
        assert!(s.chat.entries.is_empty());
    }

    #[test]
    fn session_ready_connects() {
        let mut s = AppSession::new();
        s.handle_server_message(&ServerMessage::SessionReady {
            profile: "local".to_string(),
            model: "qwen3".to_string(),
            pending_proposals: vec!["p-abc".to_string()],
        });
        assert_eq!(s.connection_state, ConnectionState::Connected);
        assert_eq!(s.profile, "local");
        assert_eq!(s.model, "qwen3");
        assert_eq!(s.proposals.len(), 1);
        assert_eq!(s.proposals[0].proposal_id, "p-abc");
    }

    #[test]
    fn token_creates_assistant_entry() {
        let mut s = AppSession::new();
        s.handle_server_message(&ServerMessage::Token {
            content: "Hello".to_string(),
        });
        assert_eq!(s.chat.entries.len(), 1);
        assert!(matches!(
            s.chat.entries[0].entry_type,
            ChatEntryType::AssistantText
        ));
        assert_eq!(s.chat.entries[0].content, "Hello");
    }

    #[test]
    fn token_appends_to_existing_assistant() {
        let mut s = AppSession::new();
        s.chat.add_user_message("hi");
        s.chat.add_assistant_text("I'll");
        s.handle_server_message(&ServerMessage::Token {
            content: " help".to_string(),
        });
        assert_eq!(s.chat.entries.len(), 2);
        assert_eq!(s.chat.entries[1].content, "I'll help");
    }

    #[test]
    fn tool_call_creates_tool_entry() {
        let mut s = AppSession::new();
        s.handle_server_message(&ServerMessage::ToolCall {
            id: "tc-1".to_string(),
            name: "inspect".to_string(),
            args: serde_json::json!({"target": "system"}),
            status: ToolCallStatus::Running,
        });
        assert_eq!(s.chat.entries.len(), 1);
        assert!(matches!(s.chat.entries[0].entry_type, ChatEntryType::ToolCall));
        assert_eq!(s.chat.entries[0].tool_id.as_deref(), Some("tc-1"));
        assert!(matches!(
            s.turn_state,
            TurnState::ToolRunning { .. }
        ));
    }

    #[test]
    fn tool_result_resolves_tool_call() {
        let mut s = AppSession::new();
        s.chat.add_tool_call("tc-1", "inspect", serde_json::json!({}));
        s.handle_server_message(&ServerMessage::ToolResult {
            id: "tc-1".to_string(),
            name: "inspect".to_string(),
            output: "CPU: 8 cores".to_string(),
            success: true,
        });
        assert!(matches!(s.chat.entries[0].entry_type, ChatEntryType::ToolResult));
        assert_eq!(s.chat.entries[0].content, "CPU: 8 cores");
        assert_eq!(s.chat.entries[0].tool_success, Some(true));
    }

    #[test]
    fn approval_request_creates_entry() {
        let mut s = AppSession::new();
        s.handle_server_message(&ServerMessage::ApprovalRequest {
            proposal_id: "p-abc".to_string(),
            summary: "Install nginx".to_string(),
            tool_call_id: "tc-2".to_string(),
        });
        assert_eq!(s.chat.entries.len(), 1);
        assert!(matches!(s.chat.entries[0].entry_type, ChatEntryType::ApprovalRequest));
        assert_eq!(s.chat.entries[0].proposal_id.as_deref(), Some("p-abc"));
        assert!(matches!(s.turn_state, TurnState::AwaitingApproval));
    }

    #[test]
    fn turn_complete_ends_turn() {
        let mut s = AppSession::new();
        s.turn_state = TurnState::Streaming;
        s.chat.add_assistant_text("Hello");
        s.handle_server_message(&ServerMessage::TurnComplete {
            content: "Hello".to_string(),
        });
        assert!(matches!(s.turn_state, TurnState::Completed));
        assert_eq!(s.chat.entries.len(), 1);
    }

    #[test]
    fn error_sets_error_state() {
        let mut s = AppSession::new();
        s.handle_server_message(&ServerMessage::Error {
            message: "LLM unreachable".to_string(),
        });
        assert!(matches!(s.turn_state, TurnState::Error { .. }));
        if let TurnState::Error { message } = &s.turn_state {
            assert_eq!(message, "LLM unreachable");
        }
    }

    #[test]
    fn status_response_parses_fields() {
        let mut s = AppSession::new();
        s.handle_server_message(&ServerMessage::StatusResponse {
            target: "system".to_string(),
            data: serde_json::json!({
                "output": "CPU: 8 cores\nRAM: 16 GB\nDisk: 120 GB free"
            }),
        });
        assert!(s.status.cpu_info.contains("CPU"));
        assert!(s.status.ram_used.contains("RAM"));
        assert!(s.status.disk_used.contains("Disk"));
    }

    #[test]
    fn proposal_created_event_adds_proposal() {
        let mut s = AppSession::new();
        s.handle_server_message(&ServerMessage::Event {
            event: "proposal_created".to_string(),
            data: serde_json::json!({
                "proposal_id": "p-new",
                "summary": "Install htop"
            }),
        });
        assert_eq!(s.proposals.len(), 1);
        assert_eq!(s.proposals[0].proposal_id, "p-new");
        assert_eq!(s.proposals[0].summary, "Install htop");
    }

    #[test]
    fn audit_entry_event_prepends() {
        let mut s = AppSession::new();
        s.handle_server_message(&ServerMessage::Event {
            event: "audit_entry".to_string(),
            data: serde_json::json!({
                "id": "a-1",
                "timestamp": "now",
                "action": {"type": "Inspect"},
                "summary": "Inspected system",
                "result": {"status": "Success"},
                "prompt": "check system",
                "actor": "agent"
            }),
        });
        assert_eq!(s.audit.len(), 1);
        assert_eq!(s.audit[0].audit_id, "a-1");
    }

    #[test]
    fn watchdog_alert_increments_count() {
        let mut s = AppSession::new();
        s.handle_server_message(&ServerMessage::Event {
            event: "watchdog_alert".to_string(),
            data: serde_json::json!({"check": "disk", "severity": "warning"}),
        });
        assert_eq!(s.status.watchdog_alert_count, 1);
    }

    #[test]
    fn pure_thinking_token_is_filtered() {
        let mut s = AppSession::new();
        s.handle_server_message(&ServerMessage::Token {
            content: "<think>Let me reason about this".to_string(),
        });
        assert!(s.chat.entries.is_empty());
        assert!(matches!(s.turn_state, TurnState::Thinking));
    }

    #[test]
    fn close_thinking_tag_strips_content() {
        let mut s = AppSession::new();
        s.in_thinking = true;
        s.turn_state = TurnState::Thinking;
        // Token that ends thinking block - stripped content should be empty
        s.handle_server_message(&ServerMessage::Token {
            content: "</think>".to_string(),
        });
        assert!(s.chat.entries.is_empty());
        assert!(matches!(s.turn_state, TurnState::Thinking));
    }

    #[test]
    fn content_after_thinking_becomes_entry() {
        let mut s = AppSession::new();
        s.in_thinking = true;
        s.turn_state = TurnState::Thinking;
        // Token that ends thinking AND has content
        s.handle_server_message(&ServerMessage::Token {
            content: "</think> The answer is".to_string(),
        });
        assert_eq!(s.chat.entries.len(), 1);
        assert!(matches!(s.chat.entries[0].entry_type, ChatEntryType::AssistantText));
        assert!(s.chat.entries[0].content.contains("The answer is"));
        assert!(matches!(s.turn_state, TurnState::Streaming));
    }

    #[test]
    fn thinking_open_in_midst_of_content() {
        let mut s = AppSession::new();
        // Token with content followed by thinking
        s.handle_server_message(&ServerMessage::Token {
            content: "Here's my reasoning<think>".to_string(),
        });
        assert_eq!(s.chat.entries.len(), 1);
        assert_eq!(s.chat.entries[0].content, "Here's my reasoning");
        assert!(s.in_thinking);
        assert!(matches!(s.turn_state, TurnState::Thinking));
    }

    #[test]
    fn multiple_thinking_tokens_all_filtered() {
        let mut s = AppSession::new();
        s.handle_server_message(&ServerMessage::Token {
            content: "<think>".to_string(),
        });
        s.handle_server_message(&ServerMessage::Token {
            content: "First reasoning step".to_string(),
        });
        s.handle_server_message(&ServerMessage::Token {
            content: "Second reasoning step".to_string(),
        });
        s.handle_server_message(&ServerMessage::Token {
            content: "</think>".to_string(),
        });
        assert!(s.chat.entries.is_empty());
        assert!(s.in_thinking == false);
    }

    #[test]
    fn thinking_then_real_content() {
        let mut s = AppSession::new();
        s.handle_server_message(&ServerMessage::Token {
            content: "<think>Let me reason</think>".to_string(),
        });
        // First token is pure thinking - no entry
        assert_eq!(s.chat.entries.len(), 0);

        s.handle_server_message(&ServerMessage::Token {
            content: " The answer is 42".to_string(),
        });
        // Second token has content - creates entry
        assert_eq!(s.chat.entries.len(), 1);
        assert_eq!(s.chat.entries[0].content, " The answer is 42");
        assert!(matches!(s.turn_state, TurnState::Streaming));
    }
}
