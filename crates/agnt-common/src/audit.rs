use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fs::{self, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::Path;

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

    /// Load all entries from a JSONL file.
    pub fn load(path: impl AsRef<Path>) -> Result<Self, String> {
        let path_str = path.as_ref().display().to_string();
        let file = match fs::File::open(path.as_ref()) {
            Ok(f) => f,
            Err(_) => return Ok(Self::new(path_str)),
        };
        let reader = BufReader::new(file);
        let mut entries = Vec::new();
        for line in reader.lines() {
            let line = line.map_err(|e| format!("Failed to read audit log: {}", e))?;
            if line.trim().is_empty() {
                continue;
            }
            match serde_json::from_str(&line) {
                Ok(entry) => entries.push(entry),
                Err(e) => {
                    eprintln!("Warning: skipping malformed audit entry: {}", e);
                }
            }
        }
        Ok(Self {
            entries,
            log_path: path_str,
        })
    }

    /// Append a single entry to the JSONL file on disk.
    pub fn append_to_disk(path: impl AsRef<Path>, entry: &AuditEntry) -> Result<(), String> {
        if let Some(parent) = path.as_ref().parent() {
            fs::create_dir_all(parent)
                .map_err(|e| format!("Failed to create audit log dir: {}", e))?;
        }
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(path.as_ref())
            .map_err(|e| format!("Failed to open audit log: {}", e))?;
        let json = serde_json::to_string(entry)
            .map_err(|e| format!("Failed to serialize audit entry: {}", e))?;
        writeln!(file, "{}", json)
            .map_err(|e| format!("Failed to write audit entry: {}", e))?;
        Ok(())
    }

    /// Get the most recent N entries.
    pub fn recent(&self, limit: usize) -> Vec<&AuditEntry> {
        self.entries.iter().rev().take(limit).collect()
    }
}

/// Generate a short audit entry ID.
pub fn audit_id() -> String {
    let ts = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    format!("a-{:x}", ts)
}
