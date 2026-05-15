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

pub fn execute_search(
    query: &str,
    limit: usize,
    json: bool,
    config_dir: Option<&PathBuf>,
) -> Result<String, String> {
    let log_path = get_log_path(config_dir);
    let log = AuditLog::load(&log_path)?;

    let q = query.to_lowercase();
    let mut matched: Vec<&AuditEntry> = log
        .entries
        .iter()
        .filter(|e| {
            e.summary.to_lowercase().contains(&q)
                || e.prompt
                    .as_deref()
                    .map_or(false, |p| p.to_lowercase().contains(&q))
                || e.rationale
                    .as_deref()
                    .map_or(false, |r| r.to_lowercase().contains(&q))
                || e.files_changed
                    .iter()
                    .any(|f| f.to_lowercase().contains(&q))
                || e.files_written
                    .iter()
                    .any(|f| f.to_lowercase().contains(&q))
                || e.files_deleted
                    .iter()
                    .any(|f| f.to_lowercase().contains(&q))
                || e.id.to_lowercase().contains(&q)
        })
        .collect();

    matched.reverse();
    matched.truncate(limit);

    if matched.is_empty() {
        return Ok(format!("No audit entries found matching '{}'.\n", query));
    }

    if json {
        return serde_json::to_string_pretty(&matched)
            .map_err(|e| format!("Failed to serialize: {}", e));
    }

    let mut out = String::new();
    out.push_str(&format!(
        "Audit entries matching '{}' ({} shown):\n\n",
        query,
        matched.len()
    ));
    for entry in matched {
        let ts = entry.timestamp.format("%Y-%m-%d %H:%M:%S");
        let status = match entry.result {
            AuditResult::Success { .. } => "OK",
            AuditResult::Failed { .. } => "FAIL",
            AuditResult::Pending { .. } => "PENDING",
        };
        let prompt_snippet = entry
            .prompt
            .as_deref()
            .map(|p| {
                let truncated: String = p.chars().take(50).collect();
                if p.len() > 50 {
                    format!("{}...", truncated)
                } else {
                    truncated
                }
            })
            .map(|s| format!(" | Prompt: {}", s))
            .unwrap_or_default();
        out.push_str(&format!(
            "  {} | {} | {:8} | {}{}\n",
            entry.id, ts, status, entry.summary, prompt_snippet
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

    let prompt_line = entry
        .prompt
        .as_deref()
        .map(|p| format!("\n         Prompt: {}\n", p))
        .unwrap_or_default();

    Ok(format!(
        "Entry: {}\n\
         Time:  {}\n\
         Actor: {}\n\
         Status: {}\n\
         Summary: {}\n\
         {}Files changed:\n{}\n\
         Rollback: {}\n",
        entry.id, ts, entry.actor, status, entry.summary, prompt_line, files, rollback,
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
        files_written: vec![],
        files_deleted: vec![],
        rollback_hint: None,
        prompt: None,
        rationale: None,
        result: AuditResult::Success { message: None },
    };
    let log_path = get_log_path(config_dir);
    AuditLog::append_to_disk(&log_path, &entry)
}

#[allow(dead_code)]
pub fn log_write(path: &str, bytes: usize, config_dir: Option<&PathBuf>) {
    let entry = AuditEntry {
        id: audit_id(),
        timestamp: Utc::now(),
        action: AuditAction::Generic {
            description: "write_file".to_string(),
        },
        actor: "user".to_string(),
        summary: format!("Wrote {} ({} bytes)", path, bytes),
        files_changed: vec![path.to_string()],
        files_written: vec![path.to_string()],
        files_deleted: vec![],
        rollback_hint: None,
        prompt: None,
        rationale: None,
        result: AuditResult::Success { message: None },
    };
    let log_path = get_log_path(config_dir);
    let _ = AuditLog::append_to_disk(&log_path, &entry);
}

#[allow(dead_code)]
pub fn log_edit(path: &str, old_str: &str, new_str: &str, config_dir: Option<&PathBuf>) {
    let entry = AuditEntry {
        id: audit_id(),
        timestamp: Utc::now(),
        action: AuditAction::Generic {
            description: "edit_file".to_string(),
        },
        actor: "user".to_string(),
        summary: format!(
            "Edited {}: '{}' → '{}'",
            path,
            old_str.chars().take(60).collect::<String>(),
            new_str.chars().take(60).collect::<String>(),
        ),
        files_changed: vec![path.to_string()],
        files_written: vec![path.to_string()],
        files_deleted: vec![],
        rollback_hint: None,
        prompt: None,
        rationale: None,
        result: AuditResult::Success { message: None },
    };
    let log_path = get_log_path(config_dir);
    let _ = AuditLog::append_to_disk(&log_path, &entry);
}

#[allow(dead_code)]
pub fn log_bash(
    command: &str,
    exit_code: i32,
    stdout: &str,
    stderr: &str,
    config_dir: Option<&PathBuf>,
) {
    let excerpt: String = if !stdout.is_empty() {
        stdout.chars().take(80).collect()
    } else if !stderr.is_empty() {
        stderr.chars().take(80).collect()
    } else {
        String::new()
    };

    let status = if exit_code == 0 {
        AuditResult::Success { message: None }
    } else {
        AuditResult::Failed {
            error: format!("exit {}", exit_code),
        }
    };

    let entry = AuditEntry {
        id: audit_id(),
        timestamp: Utc::now(),
        action: AuditAction::Generic {
            description: "run_bash".to_string(),
        },
        actor: "user".to_string(),
        summary: format!("bash: {} (exit {}) — {}", command, exit_code, excerpt,),
        files_changed: Vec::new(),
        files_written: vec![],
        files_deleted: vec![],
        rollback_hint: None,
        prompt: None,
        rationale: None,
        result: status,
    };
    let log_path = get_log_path(config_dir);
    let _ = AuditLog::append_to_disk(&log_path, &entry);
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

    #[test]
    fn test_provenance_roundtrip() {
        use agnt_common::audit::AuditLog;
        use agnt_common::audit::{audit_id, AuditAction, AuditResult};
        use chrono::Utc;

        let dir = PathBuf::from("/tmp/agntos-audit-provenance");
        let _ = std::fs::remove_dir_all(&dir);
        let log_path = dir.join("audit.jsonl");

        let entry = AuditEntry {
            id: audit_id(),
            timestamp: Utc::now(),
            action: AuditAction::Apply {
                proposal_id: "p-test".into(),
            },
            actor: "agent".into(),
            summary: "Applied: Install htop".into(),
            files_changed: vec!["packages/htop.nix".into()],
            files_written: vec!["packages/htop.nix".into()],
            files_deleted: vec![],
            rollback_hint: None,
            result: AuditResult::Success { message: None },
            prompt: Some("Install htop so I can monitor memory".into()),
            rationale: None,
        };

        // Write
        AuditLog::append_to_disk(&log_path, &entry).unwrap();

        // Read back
        let log = AuditLog::load(&log_path).unwrap();
        assert_eq!(log.entries.len(), 1);
        assert_eq!(
            log.entries[0].prompt.as_deref(),
            Some("Install htop so I can monitor memory")
        );

        // Verify show output includes provenance
        let show_out = execute_show(&entry.id, false, Some(&dir)).unwrap();
        assert!(show_out.contains("Install htop so I can monitor memory"));
        assert!(show_out.contains("Prompt:"));

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_provenance_backward_compat() {
        use agnt_common::audit::AuditLog;

        let dir = PathBuf::from("/tmp/agntos-audit-backcompat");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let log_path = dir.join("audit.jsonl");

        // Write a legacy entry without prompt/rationale fields
        let legacy = r#"{"id":"a-legacy","timestamp":"2026-01-01T00:00:00Z","action":{"type":"Apply","proposal_id":"p-old"},"actor":"user","summary":"Old style","files_changed":[],"files_written":[],"files_deleted":[],"rollback_hint":null,"result":{"status":"Success","message":null}}"#;
        std::fs::write(&log_path, format!("{}\n", legacy)).unwrap();

        let log = AuditLog::load(&log_path).unwrap();
        assert_eq!(log.entries.len(), 1);
        // Default should be None, not an error
        assert!(log.entries[0].prompt.is_none());
        assert!(log.entries[0].rationale.is_none());

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_search_by_prompt() {
        use agnt_common::audit::{audit_id, AuditAction, AuditResult};
        use chrono::Utc;

        let dir = PathBuf::from("/tmp/agntos-audit-search");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let log_path = dir.join("audit.jsonl");

        // Write two entries — one with a relevant prompt, one without
        let entry_htop = AuditEntry {
            id: audit_id(),
            timestamp: Utc::now(),
            action: AuditAction::Apply {
                proposal_id: "p1".into(),
            },
            actor: "agent".into(),
            summary: "Applied: Install htop".into(),
            files_changed: vec!["packages/htop.nix".into()],
            files_written: vec!["packages/htop.nix".into()],
            files_deleted: vec![],
            rollback_hint: None,
            result: AuditResult::Success { message: None },
            prompt: Some("Install htop so I can monitor memory usage".into()),
            rationale: None,
        };
        AuditLog::append_to_disk(&log_path, &entry_htop).unwrap();

        let entry_firefox = AuditEntry {
            id: audit_id(),
            timestamp: Utc::now(),
            action: AuditAction::Apply {
                proposal_id: "p2".into(),
            },
            actor: "agent".into(),
            summary: "Applied: Install firefox".into(),
            files_changed: vec!["packages/firefox.nix".into()],
            files_written: vec!["packages/firefox.nix".into()],
            files_deleted: vec![],
            rollback_hint: None,
            result: AuditResult::Success { message: None },
            prompt: Some("Install a web browser".into()),
            rationale: None,
        };
        AuditLog::append_to_disk(&log_path, &entry_firefox).unwrap();

        // Search by prompt content
        let result = execute_search("monitor memory", 10, false, Some(&dir)).unwrap();
        assert!(
            result.contains("htop"),
            "should find htop entry: {}",
            result
        );
        assert!(
            !result.contains("firefox"),
            "should not find firefox entry: {}",
            result
        );

        // Search by summary
        let result = execute_search("firefox", 10, false, Some(&dir)).unwrap();
        assert!(
            result.contains("firefox"),
            "should find firefox entry: {}",
            result
        );

        // Search by file path
        let result = execute_search("htop.nix", 10, false, Some(&dir)).unwrap();
        assert!(
            result.contains("htop"),
            "should find by file path: {}",
            result
        );

        // Search with no matches
        let result = execute_search("nonexistent", 10, false, Some(&dir)).unwrap();
        assert!(
            result.contains("No audit entries found"),
            "should report no matches: {}",
            result
        );

        let _ = std::fs::remove_dir_all(&dir);
    }
}
