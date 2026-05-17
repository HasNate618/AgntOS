use serde_json::Value;

#[derive(Debug, Clone)]
pub enum ChatEntryType {
    UserMessage,
    AssistantText,
    ToolCall,
    ToolResult,
    ApprovalRequest,
}

#[derive(Debug, Clone)]
pub struct ChatEntry {
    pub entry_type: ChatEntryType,
    pub content: String,
    pub tool_name: Option<String>,
    pub tool_id: Option<String>,
    pub tool_args: Option<Value>,
    pub tool_status: Option<String>,
    pub tool_success: Option<bool>,
    pub proposal_id: Option<String>,
    pub proposal_summary: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ChatModel {
    pub entries: Vec<ChatEntry>,
}

impl Default for ChatModel {
    fn default() -> Self {
        Self::new()
    }
}

impl ChatModel {
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    pub fn add_user_message(&mut self, content: &str) {
        self.entries.push(ChatEntry {
            entry_type: ChatEntryType::UserMessage,
            content: content.to_string(),
            tool_name: None,
            tool_id: None,
            tool_args: None,
            tool_status: None,
            tool_success: None,
            proposal_id: None,
            proposal_summary: None,
        });
    }

    pub fn add_assistant_text(&mut self, content: &str) {
        self.entries.push(ChatEntry {
            entry_type: ChatEntryType::AssistantText,
            content: content.to_string(),
            tool_name: None,
            tool_id: None,
            tool_args: None,
            tool_status: None,
            tool_success: None,
            proposal_id: None,
            proposal_summary: None,
        });
    }

    pub fn append_token(&mut self, content: &str) {
        if let Some(entry) = self.entries.last_mut() {
            if matches!(entry.entry_type, ChatEntryType::AssistantText) {
                entry.content.push_str(content);
            }
        }
    }

    pub fn add_tool_call(&mut self, id: &str, name: &str, args: Value) {
        self.entries.push(ChatEntry {
            entry_type: ChatEntryType::ToolCall,
            content: String::new(),
            tool_name: Some(name.to_string()),
            tool_id: Some(id.to_string()),
            tool_args: Some(args),
            tool_status: Some("running".to_string()),
            tool_success: None,
            proposal_id: None,
            proposal_summary: None,
        });
    }

    pub fn resolve_tool_call(&mut self, id: &str, output: &str, success: bool) {
        if let Some(entry) = self
            .entries
            .iter_mut()
            .rev()
            .find(|e| e.tool_id.as_deref() == Some(id))
        {
            entry.entry_type = ChatEntryType::ToolResult;
            entry.content = output.to_string();
            entry.tool_status = Some("done".to_string());
            entry.tool_success = Some(success);
        }
    }

    pub fn add_approval_request(&mut self, proposal_id: &str, summary: &str, tool_call_id: &str) {
        self.entries.push(ChatEntry {
            entry_type: ChatEntryType::ApprovalRequest,
            content: String::new(),
            tool_name: Some("apply".to_string()),
            tool_id: Some(tool_call_id.to_string()),
            tool_args: None,
            tool_status: None,
            tool_success: None,
            proposal_id: Some(proposal_id.to_string()),
            proposal_summary: Some(summary.to_string()),
        });
    }

    pub fn clear(&mut self) {
        self.entries.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn chat_model_starts_empty() {
        let m = ChatModel::new();
        assert!(m.entries.is_empty());
    }

    #[test]
    fn chat_model_add_user_message() {
        let mut m = ChatModel::new();
        m.add_user_message("hello");
        assert_eq!(m.entries.len(), 1);
        assert!(matches!(
            m.entries[0].entry_type,
            ChatEntryType::UserMessage
        ));
        assert_eq!(m.entries[0].content, "hello");
    }

    #[test]
    fn chat_model_append_token() {
        let mut m = ChatModel::new();
        m.add_assistant_text("I'll");
        m.append_token(" install");
        m.append_token(" nginx");
        assert_eq!(m.entries[0].content, "I'll install nginx");
    }

    #[test]
    fn chat_model_tool_call_resolve() {
        let mut m = ChatModel::new();
        m.add_tool_call("tc-1", "inspect", serde_json::json!({"target": "system"}));
        assert!(matches!(m.entries[0].entry_type, ChatEntryType::ToolCall));

        m.resolve_tool_call("tc-1", "CPU: 8 cores", true);
        assert!(matches!(m.entries[0].entry_type, ChatEntryType::ToolResult));
        assert_eq!(m.entries[0].content, "CPU: 8 cores");
        assert_eq!(m.entries[0].tool_success, Some(true));
    }

    #[test]
    fn chat_model_approval_request() {
        let mut m = ChatModel::new();
        m.add_approval_request("p-abc", "Install nginx", "tc-2");
        assert!(matches!(
            m.entries[0].entry_type,
            ChatEntryType::ApprovalRequest
        ));
        assert_eq!(m.entries[0].proposal_id.as_deref(), Some("p-abc"));
        assert_eq!(
            m.entries[0].proposal_summary.as_deref(),
            Some("Install nginx")
        );
    }

    #[test]
    fn chat_model_clear() {
        let mut m = ChatModel::new();
        m.add_user_message("test");
        m.clear();
        assert!(m.entries.is_empty());
    }
}
