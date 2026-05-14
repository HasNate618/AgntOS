use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEntry {
    pub id: String,
    pub timestamp: DateTime<Utc>,
    pub action: AuditAction,
    pub actor: String,
    pub summary: String,
    pub files_changed: Vec<String>,
    pub rollback_hint: Option<String>,
    pub result: AuditResult,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum AuditAction {
    Inspect { target: String },
    Propose { target: String },
    Apply { proposal_id: String },
    Rollback { generation_id: String },
    ModelConfig { change: String },
    Generic { description: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "status")]
pub enum AuditResult {
    Success { message: Option<String> },
    Failed { error: String },
    Pending { reason: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditLog {
    pub entries: Vec<AuditEntry>,
    pub log_path: String,
}

impl AuditLog {
    pub fn new(log_path: String) -> Self {
        Self {
            entries: Vec::new(),
            log_path,
        }
    }

    pub fn append(&mut self, entry: AuditEntry) {
        self.entries.push(entry);
    }
}
