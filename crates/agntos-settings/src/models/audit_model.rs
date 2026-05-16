#[derive(Debug, Clone)]
pub struct AuditEntry {
    pub audit_id: String,
    pub timestamp: String,
    pub action_type: String,
    pub summary: String,
    pub status: String,
    pub actor: String,
    pub prompt: String,
    pub rationale: String,
    pub files_changed: Vec<String>,
    pub rollback_hint: Option<String>,
    pub result_message: Option<String>,
}

#[derive(Debug, Clone)]
pub struct AuditModel {
    pub entries: Vec<AuditEntry>,
}

impl AuditModel {
    pub fn new() -> Self {
        Self { entries: Vec::new() }
    }

    pub fn load_from_json(&mut self, json_entries: Vec<serde_json::Value>) {
        self.entries.clear();
        for val in json_entries {
            let obj = match val.as_object() {
                Some(o) => o,
                None => continue,
            };

            let audit_id = obj.get("id").and_then(|v| v.as_str()).unwrap_or("?").to_string();
            let summary = obj.get("summary").and_then(|v| v.as_str()).unwrap_or("").to_string();
            let status = obj.get("result")
                .and_then(|r| r.get("status"))
                .and_then(|s| s.as_str())
                .unwrap_or("unknown")
                .to_string();

            let action_type = obj.get("action")
                .and_then(|a| a.get("type"))
                .and_then(|t| t.as_str())
                .unwrap_or("")
                .to_string();

            let prompt = obj.get("prompt").and_then(|v| v.as_str()).unwrap_or("").to_string();
            let rationale = obj.get("rationale").and_then(|v| v.as_str()).unwrap_or("").to_string();
            let files = obj.get("files_changed")
                .and_then(|f| f.as_array())
                .map(|arr| arr.iter().filter_map(|v| v.as_str().map(String::from)).collect())
                .unwrap_or_default();
            let rollback_hint = obj.get("rollback_hint").and_then(|v| v.as_str()).map(String::from);
            let result_msg = obj.get("result")
                .and_then(|r| r.get("message"))
                .and_then(|v| v.as_str())
                .map(String::from);
            let actor = obj.get("actor").and_then(|v| v.as_str()).unwrap_or("").to_string();
            let timestamp = obj.get("timestamp").and_then(|v| v.as_str()).unwrap_or("").to_string();

            self.entries.push(AuditEntry {
                audit_id,
                timestamp,
                action_type,
                summary,
                status,
                actor,
                prompt,
                rationale,
                files_changed: files,
                rollback_hint,
                result_message: result_msg,
            });
        }
    }

    pub fn clear(&mut self) {
        self.entries.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn audit_model_starts_empty() {
        let m = AuditModel::new();
        assert!(m.entries.is_empty());
    }

    #[test]
    fn audit_model_load_entries() {
        let mut m = AuditModel::new();
        let entries = vec![
            serde_json::json!({
                "id": "a-001",
                "timestamp": "2025-05-16T14:30:00Z",
                "action": {"type": "Apply", "proposal_id": "p-abc"},
                "summary": "Applied: Install nginx",
                "result": {"status": "Success", "message": "Rebuild ok"},
                "actor": "agent",
                "prompt": "install nginx",
                "files_changed": ["/etc/agntos/generated/packages.nix"],
            }),
        ];
        m.load_from_json(entries);
        assert_eq!(m.entries.len(), 1);
        assert_eq!(m.entries[0].audit_id, "a-001");
        assert_eq!(m.entries[0].summary, "Applied: Install nginx");
        assert_eq!(m.entries[0].status, "Success");
        assert_eq!(m.entries[0].action_type, "Apply");
        assert_eq!(m.entries[0].prompt, "install nginx");
    }

    #[test]
    fn audit_model_load_skips_non_objects() {
        let mut m = AuditModel::new();
        m.load_from_json(vec![
            serde_json::json!("not an object"),
        ]);
        assert!(m.entries.is_empty());
    }

    #[test]
    fn audit_model_clear() {
        let mut m = AuditModel::new();
        m.load_from_json(vec![
            serde_json::json!({"id": "a-001", "action": {"type": "Apply"}, "summary": "test", "result": {"status": "Success"}}),
        ]);
        assert_eq!(m.entries.len(), 1);
        m.clear();
        assert!(m.entries.is_empty());
    }
}
