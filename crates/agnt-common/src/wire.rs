use serde::{Deserialize, Serialize};

// ── Client → Server ──────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ClientMessage {
    #[serde(rename = "init")]
    Init {
        #[serde(default)]
        config_dir: Option<String>,
    },
    #[serde(rename = "chat")]
    Chat { prompt: String },
    #[serde(rename = "approve")]
    Approve { proposal_id: String },
    #[serde(rename = "dismiss")]
    Dismiss {
        proposal_id: String,
        #[serde(default)]
        reason: Option<String>,
    },
    #[serde(rename = "status")]
    Status {
        #[serde(default = "default_system")]
        target: String,
    },
    #[serde(rename = "audit")]
    Audit {
        action: AuditRequestAction,
        #[serde(default)]
        query: Option<String>,
        #[serde(default)]
        id: Option<String>,
        #[serde(default = "default_limit")]
        limit: u32,
    },
    #[serde(rename = "cancel")]
    Cancel {},
}

fn default_system() -> String {
    "system".to_string()
}
fn default_limit() -> u32 {
    50
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "action")]
pub enum AuditRequestAction {
    #[serde(rename = "list")]
    List,
    #[serde(rename = "search")]
    Search,
    #[serde(rename = "show")]
    Show,
}

// ── Server → Client ──────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ServerMessage {
    #[serde(rename = "session_ready")]
    SessionReady {
        profile: String,
        model: String,
        #[serde(default)]
        pending_proposals: Vec<String>,
    },
    #[serde(rename = "status_response")]
    StatusResponse {
        target: String,
        data: serde_json::Value,
    },
    #[serde(rename = "token")]
    Token { content: String },
    #[serde(rename = "tool_call")]
    ToolCall {
        id: String,
        name: String,
        args: serde_json::Value,
        status: ToolCallStatus,
    },
    #[serde(rename = "tool_result")]
    ToolResult {
        id: String,
        name: String,
        output: String,
        success: bool,
    },
    #[serde(rename = "approval_request")]
    ApprovalRequest {
        proposal_id: String,
        summary: String,
        tool_call_id: String,
    },
    #[serde(rename = "turn_complete")]
    TurnComplete { content: String },
    #[serde(rename = "audit_response")]
    AuditResponse {
        entries: Vec<serde_json::Value>,
    },
    #[serde(rename = "event")]
    Event {
        event: String,
        data: serde_json::Value,
    },
    #[serde(rename = "error")]
    Error { message: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "status")]
pub enum ToolCallStatus {
    #[serde(rename = "running")]
    Running,
    #[serde(rename = "done")]
    Done,
}

// ── Backward compat ──────────────────────────────────────────────────────

/// Detect whether a raw JSON string is a legacy one-shot `{"prompt": "..."}` message
/// or a new typed `{"type": "..."}`  message.
pub fn is_legacy_prompt(raw: &str) -> bool {
    raw.contains("\"prompt\"") && !raw.contains("\"type\"")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn client_message_init_serialization() {
        let msg = ClientMessage::Init { config_dir: None };
        let json = serde_json::to_string(&msg).unwrap();
        assert!(json.contains("\"type\":\"init\""));
    }

    #[test]
    fn client_message_chat_serialization() {
        let msg = ClientMessage::Chat { prompt: "install nginx".to_string() };
        let json = serde_json::to_string(&msg).unwrap();
        assert!(json.contains("\"type\":\"chat\""));
        assert!(json.contains("install nginx"));
    }

    #[test]
    fn server_message_token_deserialization() {
        let json = r#"{"type":"token","content":"hello"}"#;
        let msg: ServerMessage = serde_json::from_str(json).unwrap();
        match msg {
            ServerMessage::Token { content } => assert_eq!(content, "hello"),
            _ => panic!("expected Token"),
        }
    }

    #[test]
    fn is_legacy_prompt_detection() {
        assert!(is_legacy_prompt(r#"{"prompt":"hello"}"#));
        assert!(!is_legacy_prompt(r#"{"type":"init"}"#));
        assert!(!is_legacy_prompt(r#"{"type":"chat","prompt":"hello"}"#));
    }

    #[test]
    fn roundtrip_client_messages() {
        let messages = vec![
            ClientMessage::Approve { proposal_id: "p-abc".to_string() },
            ClientMessage::Dismiss { proposal_id: "p-abc".to_string(), reason: Some("not needed".to_string()) },
            ClientMessage::Status { target: "system".to_string() },
            ClientMessage::Cancel {},
        ];
        for msg in &messages {
            let json = serde_json::to_string(msg).unwrap();
            let back: ClientMessage = serde_json::from_str(&json).unwrap();
            let json2 = serde_json::to_string(&back).unwrap();
            assert_eq!(json, json2);
        }
    }

    #[test]
    fn roundtrip_server_messages() {
        let messages = vec![
            ServerMessage::SessionReady { profile: "local".to_string(), model: "qwen".to_string(), pending_proposals: vec![] },
            ServerMessage::ToolCall { id: "tc-1".to_string(), name: "inspect".to_string(), args: serde_json::json!({}), status: ToolCallStatus::Running },
            ServerMessage::Error { message: "test".to_string() },
        ];
        for msg in &messages {
            let json = serde_json::to_string(msg).unwrap();
            let back: ServerMessage = serde_json::from_str(&json).unwrap();
            let json2 = serde_json::to_string(&back).unwrap();
            assert_eq!(json, json2);
        }
    }
}
