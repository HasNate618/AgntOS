use agnt_common::audit::{audit_id, AuditAction, AuditEntry, AuditLog, AuditResult};
use chrono::Utc;
use std::path::PathBuf;

const DEFAULT_LOG_DIR: &str = "/var/log/agntos";

pub fn get_log_path(config_dir: Option<&PathBuf>) -> PathBuf {
    config_dir
        .map(|d| d.join("audit.jsonl"))
        .unwrap_or_else(|| PathBuf::from(DEFAULT_LOG_DIR).join("audit.jsonl"))
}

pub fn execute_list(
    limit: usize,
    json: bool,
    config_dir: Option<&PathBuf>,
) -> Result<String, String> {
    let log_path = get_log_path(config_dir);
    let log = AuditLog::load(&log_path)?;

    let entries = log.recent(limit);
    if entries.is_empty() {
        return Ok("No audit entries found.\n".to_string());
    }

    if json {
        let list: Vec<&AuditEntry> = entries;
        return serde_json::to_string_pretty(&list)
            .map_err(|e| format!("Failed to serialize: {}", e));
    }

    let mut out = String::new();
    out.push_str(&format!(
        "Recent audit entries ({} shown):\n\n",
        entries.len()
    ));
    for entry in entries {
        let ts = entry.timestamp.format("%Y-%m-%d %H:%M:%S");
        let status = match entry.result {
            AuditResult::Success { .. } => "OK",
            AuditResult::Failed { .. } => "FAIL",
            AuditResult::Pending { .. } => "PENDING",
        };
        out.push_str(&format!(
            "  {} | {} | {:8} | {}\n",
            entry.id, ts, status, entry.summary
        ));
    }
    Ok(out)
}

pub fn execute_show(id: &str, json: bool, config_dir: Option<&PathBuf>) -> Result<String, String> {
    let log_path = get_log_path(config_dir);
    let log = AuditLog::load(&log_path)?;

    let entry = log
        .entries
        .iter()
        .find(|e| e.id == id)
        .ok_or_else(|| format!("Audit entry not found: {}", id))?;

    if json {
        return serde_json::to_string_pretty(entry)
            .map_err(|e| format!("Failed to serialize: {}", e));
    }

    let ts = entry.timestamp.format("%Y-%m-%d %H:%M:%S");
    let status = match &entry.result {
        AuditResult::Success { message } => format!(
            "Success{}",
            message
                .as_ref()
                .map(|m| format!(": {}", m))
                .unwrap_or_default()
        ),
        AuditResult::Failed { error } => format!("Failed: {}", error),
        AuditResult::Pending { reason } => format!("Pending: {}", reason),
    };

    let files = if entry.files_changed.is_empty() {
        "  (none)".to_string()
    } else {
        entry
            .files_changed
            .iter()
            .map(|f| format!("  - {}", f))
            .collect::<Vec<_>>()
            .join("\n")
    };

    let rollback = entry.rollback_hint.as_deref().unwrap_or("(none)");

    Ok(format!(
        "Entry: {}\n\
         Time:  {}\n\
         Actor: {}\n\
         Status: {}\n\
         \n\
         Summary: {}\n\
         Files changed:\n{}\n\
         Rollback: {}\n",
        entry.id, ts, entry.actor, status, entry.summary, files, rollback,
    ))
}

#[allow(dead_code)]
pub fn log_inspect(target: &str, config_dir: Option<&PathBuf>) -> Result<(), String> {
    let entry = AuditEntry {
        id: audit_id(),
        timestamp: Utc::now(),
        action: AuditAction::Inspect {
            target: target.to_string(),
        },
        actor: "user".to_string(),
        summary: format!("Inspected {}", target),
        files_changed: Vec::new(),
        rollback_hint: None,
        result: AuditResult::Success { message: None },
    };
    let log_path = get_log_path(config_dir);
    AuditLog::append_to_disk(&log_path, &entry)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_execute_list_no_entries() {
        let result = execute_list(
            10,
            false,
            Some(&PathBuf::from("/tmp/nonexistent-agntos-test")),
        );
        assert!(result.is_ok());
        assert!(result.unwrap().contains("No audit entries"));
    }

    #[test]
    fn test_execute_list_empty_json() {
        let result = execute_list(
            10,
            true,
            Some(&PathBuf::from("/tmp/nonexistent-agntos-test")),
        );
        assert!(result.is_ok());
    }

    #[test]
    fn test_show_not_found() {
        let result = execute_show(
            "nonexistent",
            false,
            Some(&PathBuf::from("/tmp/agntos-audit-test")),
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_log_and_read() {
        let dir = PathBuf::from("/tmp/agntos-audit-test-write");
        let _ = std::fs::remove_dir_all(&dir);

        log_inspect("cpu", Some(&dir)).unwrap();

        let result = execute_list(10, false, Some(&dir)).unwrap();
        assert!(result.contains("Inspected cpu"));

        let _ = std::fs::remove_dir_all(&dir);
    }
}
