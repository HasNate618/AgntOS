//! `agntctl rollback` — list NixOS generations and roll back to the previous one.
//!
//! Detects flake vs channel rebuild contexts the same way as `agntctl apply`
//! (via `/etc/agntos/flake-info`).

use agnt_common::audit::{audit_id, AuditAction, AuditEntry, AuditLog, AuditResult};
use chrono::Utc;
use std::path::PathBuf;

const DEFAULT_CONFIG_DIR: &str = "/etc/agntos";

/// Returns a `nixos-rebuild` command configured for listing generations
/// (channel or flake depending on `/etc/agntos/flake-info`).
fn list_cmd(config_dir: &PathBuf) -> std::process::Command {
    let mut cmd = std::process::Command::new("nixos-rebuild");
    let flake_path = config_dir.join("flake-info");
    if flake_path.exists() {
        if let Ok(flake_ref) = std::fs::read_to_string(&flake_path) {
            let trimmed = flake_ref.trim().to_string();
            if !trimmed.is_empty() {
                cmd.arg("list-generations")
                    .arg("--flake")
                    .arg(&trimmed)
                    .arg("--impure");
                return cmd;
            }
        }
    }
    cmd.arg("list-generations");
    cmd
}

/// Returns a `nixos-rebuild` command configured for rolling back
/// (channel or flake depending on `/etc/agntos/flake-info`).
fn rollback_cmd(config_dir: &PathBuf) -> std::process::Command {
    let mut cmd = std::process::Command::new("nixos-rebuild");
    let flake_path = config_dir.join("flake-info");
    if flake_path.exists() {
        if let Ok(flake_ref) = std::fs::read_to_string(&flake_path) {
            let trimmed = flake_ref.trim().to_string();
            if !trimmed.is_empty() {
                cmd.arg("switch")
                    .arg("--rollback")
                    .arg("--flake")
                    .arg(&trimmed)
                    .arg("--impure");
                return cmd;
            }
        }
    }
    cmd.arg("switch").arg("--rollback");
    cmd
}

/// Lists NixOS generations.
pub fn execute_list(config_dir: Option<&PathBuf>) -> Result<String, String> {
    let dir = config_dir
        .cloned()
        .unwrap_or_else(|| PathBuf::from(DEFAULT_CONFIG_DIR));

    let mut cmd = list_cmd(&dir);
    let output = cmd
        .output()
        .map_err(|e| format!("Failed to run nixos-rebuild: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("nixos-rebuild returned an error:\n{}", stderr));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    Ok(stdout.to_string())
}

/// Rolls back to the previous NixOS generation and logs the action to the
/// audit log.  Returns the command log output.
pub fn execute(config_dir: Option<&PathBuf>) -> Result<String, String> {
    let dir = config_dir
        .cloned()
        .unwrap_or_else(|| PathBuf::from(DEFAULT_CONFIG_DIR));

    let mut cmd = rollback_cmd(&dir);
    let output = cmd
        .output()
        .map_err(|e| format!("Failed to run nixos-rebuild --rollback: {}", e))?;

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();

    if !output.status.success() {
        let _ = log_rollback(Err(stderr.clone()), &dir);
        return Err(format!(
            "Rollback failed:\n{}",
            if !stderr.is_empty() { &stderr } else { &stdout }
        ));
    }

    let _ = log_rollback(Ok(()), &dir);

    let mut out = String::from("Rollback succeeded.\n");
    if !stdout.is_empty() {
        for line in stdout.lines() {
            out.push_str(&format!("  {}\n", line));
        }
    }
    Ok(out)
}

/// Records the rollback action in the audit JSONL log.
fn log_rollback(result: Result<(), String>, config_dir: &PathBuf) -> Result<(), String> {
    let (audit_result, summary) = match result {
        Ok(()) => (
            AuditResult::Success { message: None },
            "Rolled back to previous NixOS generation".to_string(),
        ),
        Err(e) => (
            AuditResult::Failed { error: e.clone() },
            format!("Rollback failed: {}", e),
        ),
    };

    let entry = AuditEntry {
        id: audit_id(),
        timestamp: Utc::now(),
        action: AuditAction::Rollback {
            generation_id: "auto".to_string(),
        },
        actor: "user".to_string(),
        summary,
        files_changed: Vec::new(),
        rollback_hint: Some("Rolled back via nixos-rebuild switch --rollback".to_string()),
        result: audit_result,
    };

    let log_path = crate::audit::get_log_path(Some(config_dir));
    AuditLog::append_to_disk(&log_path, &entry)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rollback_cmd_no_flake() {
        let dir = PathBuf::from("/tmp/agntos-rollback-test-empty");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();

        let cmd = rollback_cmd(&dir);
        let args: Vec<&str> = cmd.get_args().map(|a| a.to_str().unwrap_or("")).collect();
        assert_eq!(args, vec!["switch", "--rollback"]);

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_rollback_cmd_with_flake() {
        let dir = PathBuf::from("/tmp/agntos-rollback-test-flake");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("flake-info"), "/path#config\n").unwrap();

        let cmd = rollback_cmd(&dir);
        let args: Vec<&str> = cmd.get_args().map(|a| a.to_str().unwrap_or("")).collect();
        assert_eq!(
            args,
            vec![
                "switch",
                "--rollback",
                "--flake",
                "/path#config",
                "--impure"
            ]
        );

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_list_cmd_no_flake() {
        let dir = PathBuf::from("/tmp/agntos-rollback-list-empty");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();

        let cmd = list_cmd(&dir);
        let args: Vec<&str> = cmd.get_args().map(|a| a.to_str().unwrap_or("")).collect();
        assert_eq!(args, vec!["list-generations"]);

        let _ = std::fs::remove_dir_all(&dir);
    }
}
