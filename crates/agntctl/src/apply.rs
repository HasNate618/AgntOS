use agnt_common::audit::{audit_id, AuditAction, AuditEntry, AuditLog, AuditResult};
use agnt_common::config::ConfigProposal;
use chrono::Utc;
use std::collections::HashMap;
use std::path::{Component, Path, PathBuf};

const DEFAULT_CONFIG_DIR: &str = "/etc/agntos";

/// Sanitizes a relative file path so it cannot escape `base_dir` via `..`
/// or absolute-prefix tricks.  Returns the resolved path on success.
fn resolve_safe(base: &Path, rel: &str) -> Result<PathBuf, String> {
    let candidate = Path::new(rel);
    if candidate.is_absolute() {
        return Err(format!("Refusing absolute path in proposal: {}", rel));
    }
    for component in candidate.components() {
        match component {
            Component::ParentDir => {
                return Err(format!("Path escapes config directory: {}", rel));
            }
            Component::RootDir | Component::Prefix(_) => {
                return Err(format!("Unexpected path component in: {}", rel));
            }
            _ => {}
        }
    }
    Ok(base.join(candidate))
}

/// Returns (program, args) for nixos-rebuild based on whether a flake
/// environment is detected via `/etc/agntos/flake-info`.
fn rebuild_cmd(dir: &PathBuf) -> std::process::Command {
    let mut cmd = std::process::Command::new("nixos-rebuild");
    let flake_path = dir.join("flake-info");
    if flake_path.exists() {
        if let Ok(flake_ref) = std::fs::read_to_string(&flake_path) {
            let trimmed = flake_ref.trim().to_string();
            if !trimmed.is_empty() {
                cmd.arg("test").arg("--flake").arg(&trimmed).arg("--impure");
                return cmd;
            }
        }
    }
    cmd.arg("test");
    cmd
}

pub fn execute(
    proposal_id: &str,
    dry_run: bool,
    no_rebuild: bool,
    config_dir: Option<&PathBuf>,
) -> Result<String, String> {
    let dir = config_dir
        .cloned()
        .unwrap_or_else(|| PathBuf::from(DEFAULT_CONFIG_DIR));

    // Read the proposal
    let proposal_path = dir.join("proposals").join(format!("{}.json", proposal_id));
    let proposal_json = std::fs::read_to_string(&proposal_path)
        .map_err(|_| format!("Proposal not found: {}", proposal_id))?;
    let proposal: ConfigProposal = serde_json::from_str(&proposal_json)
        .map_err(|_| format!("Invalid proposal file: {}", proposal_id))?;

    let mut out = String::new();
    out.push_str(&format!(
        "Applying proposal: {} ({})\n",
        proposal.id, proposal.summary
    ));

    // --- Snapshot existing state so we can roll back on rebuild failure ---
    let mut snapshots: HashMap<PathBuf, Option<Vec<u8>>> = HashMap::new();
    for (filename, _content) in &proposal.files_to_write {
        let fp = resolve_safe(&dir, filename)?;
        let old = if fp.exists() {
            Some(
                std::fs::read(&fp)
                    .map_err(|e| format!("Failed to snapshot {}: {}", fp.display(), e))?,
            )
        } else {
            None
        };
        snapshots.insert(fp, old);
    }
    for filename in &proposal.files_to_delete {
        let fp = resolve_safe(&dir, filename)?;
        let old = if fp.exists() {
            Some(
                std::fs::read(&fp)
                    .map_err(|e| format!("Failed to snapshot {}: {}", fp.display(), e))?,
            )
        } else {
            None
        };
        snapshots.entry(fp).or_insert(old);
    }

    // --- Helper to unwind mutations on failure ---
    let rollback_mutations = |out: &mut String| {
        for (fp, old) in snapshots.iter() {
            match old {
                Some(prev) => {
                    if let Err(e) = std::fs::write(fp, prev) {
                        *out += &format!(
                            "    (cleanup error: couldn't restore {}: {})\n",
                            fp.display(),
                            e
                        );
                    } else {
                        *out += &format!("  Restored:   {}\n", fp.display());
                    }
                }
                None => {
                    if fp.exists() {
                        let _ = std::fs::remove_file(fp);
                        *out += &format!("  Cleaned:    {}\n", fp.display());
                    }
                }
            }
        }
    };

    // Write the Nix files
    let mut written_files = Vec::new();
    for (filename, content) in &proposal.files_to_write {
        let filepath = resolve_safe(&dir, filename)?;
        if dry_run {
            out.push_str(&format!("  Would write: {}\n", filepath.display()));
        } else {
            if let Some(parent) = filepath.parent() {
                std::fs::create_dir_all(parent).map_err(|e| {
                    format!("Failed to create directory {}: {}", parent.display(), e)
                })?;
            }
            std::fs::write(&filepath, content)
                .map_err(|e| format!("Failed to write {}: {}", filepath.display(), e))?;
            out.push_str(&format!("  Written:     {}\n", filepath.display()));
        }
        written_files.push(filepath.display().to_string());
    }

    // Delete files marked for removal
    for filename in &proposal.files_to_delete {
        let filepath = resolve_safe(&dir, filename)?;
        if dry_run {
            out.push_str(&format!("  Would delete: {}\n", filepath.display()));
        } else {
            if filepath.exists() {
                std::fs::remove_file(&filepath)
                    .map_err(|e| format!("Failed to delete {}: {}", filepath.display(), e))?;
                out.push_str(&format!("  Deleted:     {}\n", filepath.display()));
                written_files.push(filepath.display().to_string());
            } else {
                out.push_str(&format!(
                    "  Skip delete: {} (not found)\n",
                    filepath.display()
                ));
            }
        }
    }

    // Run nixos-rebuild (unless --no-rebuild or --dry-run)
    if !dry_run && !no_rebuild {
        let mut rebuild_result = rebuild_cmd(&dir);
        out.push_str(&format!("\n  Running {:?}...\n", rebuild_result));
        let output = rebuild_result.output();

        match output {
            Ok(output) => {
                let stdout = String::from_utf8_lossy(&output.stdout);
                let stderr = String::from_utf8_lossy(&output.stderr);

                if output.status.success() {
                    out.push_str("  nixos-rebuild test: OK\n");
                    if !stdout.is_empty() {
                        for line in stdout.lines() {
                            out.push_str(&format!("    > {}\n", line));
                        }
                    }
                } else {
                    out.push_str(&format!(
                        "  nixos-rebuild test: FAILED (exit code: {})\n",
                        output.status
                    ));
                    if !stderr.is_empty() {
                        for line in stderr.lines() {
                            out.push_str(&format!("    ! {}\n", line));
                        }
                    }
                    // Unwind file mutations
                    rollback_mutations(&mut out);
                    // Log the failure
                    let _ = log_apply(
                        &proposal,
                        &written_files,
                        Err("nixos-rebuild failed".to_string()),
                        &dir,
                        dry_run,
                    );
                    return Err(format!(
                        "nixos-rebuild test failed. Check the audit log.\n{}",
                        out
                    ));
                }
            }
            Err(e) => {
                let msg = format!("nixos-rebuild not available: {}\n  (Install NixOS or use --no-rebuild to skip)", e);
                out.push_str(&format!("  Warning: {}\n", msg));
                let _ = log_apply(&proposal, &written_files, Err(msg.clone()), &dir, dry_run);
                return Err(out);
            }
        }
    }

    // Clean up the proposal file
    if !dry_run {
        let _ = std::fs::remove_file(&proposal_path);
        out.push_str(&format!(
            "\n  Removed proposal: {}\n",
            proposal_path.display()
        ));
    }

    // Log the apply
    let result = log_apply(&proposal, &written_files, Ok(()), &dir, dry_run);
    if let Err(e) = result {
        out.push_str(&format!("  Warning: failed to log audit entry: {}\n", e));
    }

    out.push_str(&format!(
        "\nRollback guidance:\n  {}\n",
        proposal.rollback_guidance
    ));

    if dry_run {
        out = format!("{}\nDRY RUN — nothing applied.\n", out.trim());
    }

    Ok(out)
}

fn log_apply(
    proposal: &ConfigProposal,
    files: &[String],
    result: Result<(), String>,
    config_dir: &PathBuf,
    dry_run: bool,
) -> Result<(), String> {
    if dry_run {
        return Ok(());
    }

    let (audit_result, summary) = match result {
        Ok(()) => (
            AuditResult::Success { message: None },
            format!("Applied: {}", proposal.summary),
        ),
        Err(e) => (
            AuditResult::Failed { error: e.clone() },
            format!("Failed: {} — {}", proposal.summary, e),
        ),
    };

    let entry = AuditEntry {
        id: audit_id(),
        timestamp: Utc::now(),
        action: AuditAction::Apply {
            proposal_id: proposal.id.clone(),
        },
        actor: "user".to_string(),
        summary,
        files_changed: files.to_vec(),
        rollback_hint: Some(proposal.rollback_guidance.clone()),
        result: audit_result,
    };

    let log_path = crate::audit::get_log_path(Some(config_dir));
    AuditLog::append_to_disk(&log_path, &entry)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn create_test_proposal(dir: &PathBuf, id: &str) {
        let props_dir = dir.join("proposals");
        fs::create_dir_all(&props_dir).unwrap();
        let proposal = ConfigProposal {
            id: id.to_string(),
            summary: "Test: install hello".to_string(),
            nix_changes: "environment.systemPackages = [ pkgs.hello ];".to_string(),
            files_to_write: vec![(
                "packages.nix".to_string(),
                "{ config, pkgs, ... }: {\n  environment.systemPackages = [ pkgs.hello ];\n}"
                    .to_string(),
            )],
            files_to_delete: vec![],
            rollback_guidance: "Remove hello from packages.nix.".to_string(),
        };
        let json = serde_json::to_string_pretty(&proposal).unwrap();
        fs::write(props_dir.join(format!("{}.json", id)), json).unwrap();
    }

    #[test]
    fn test_apply_missing_proposal() {
        let result = execute(
            "nonexistent",
            true,
            true,
            Some(&PathBuf::from("/tmp/agntos-apply-test")),
        );
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Proposal not found"));
    }

    #[test]
    fn test_apply_dry_run() {
        let dir = PathBuf::from("/tmp/agntos-apply-test-dry");
        let _ = fs::remove_dir_all(&dir);
        create_test_proposal(&dir, "test123");
        let result = execute("test123", true, true, Some(&dir));
        assert!(result.is_ok());
        let out = result.unwrap();
        assert!(out.contains("DRY RUN"));
        assert!(out.contains("Would write"));
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_apply_writes_files() {
        let dir = PathBuf::from("/tmp/agntos-apply-test-real");
        let _ = fs::remove_dir_all(&dir);
        create_test_proposal(&dir, "real456");
        let result = execute("real456", false, true, Some(&dir));
        assert!(result.is_ok());
        let out = result.unwrap();

        // Verify the file was written
        assert!(fs::read_to_string(dir.join("packages.nix"))
            .unwrap()
            .contains("pkgs.hello"));
        assert!(out.contains("Written"));

        // Verify proposal was cleaned up
        assert!(!dir.join("proposals").join("real456.json").exists());

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_rebuild_cmd_fallback_when_no_flake_info() {
        let dir = PathBuf::from("/tmp/agntos-apply-no-flake");
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        let cmd = rebuild_cmd(&dir);
        let args: Vec<&str> = cmd.get_args().map(|a| a.to_str().unwrap_or("")).collect();
        assert_eq!(args, vec!["test"]);
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_rebuild_cmd_detects_flake() {
        let dir = PathBuf::from("/tmp/agntos-apply-flake");
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        fs::write(dir.join("flake-info"), "/home/user/config#my-machine\n").unwrap();
        let cmd = rebuild_cmd(&dir);
        let args: Vec<&str> = cmd.get_args().map(|a| a.to_str().unwrap_or("")).collect();
        assert_eq!(
            args,
            vec![
                "test",
                "--flake",
                "/home/user/config#my-machine",
                "--impure"
            ]
        );
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_resolve_safe_rejects_absolute() {
        let base = PathBuf::from("/etc/agntos");
        assert!(resolve_safe(&base, "/etc/passwd").is_err());
    }

    #[test]
    fn test_resolve_safe_rejects_parent_dir() {
        let base = PathBuf::from("/etc/agntos");
        assert!(resolve_safe(&base, "../passwd").is_err());
        assert!(resolve_safe(&base, "packages/../../../passwd").is_err());
    }

    #[test]
    fn test_resolve_safe_allows_normal_paths() {
        let base = PathBuf::from("/etc/agntos");
        let p = resolve_safe(&base, "packages/kitty.nix").unwrap();
        assert_eq!(p, PathBuf::from("/etc/agntos/packages/kitty.nix"));
    }

    #[test]
    fn test_snapshot_rollback_writes() {
        use std::fs;
        let dir = PathBuf::from("/tmp/agntos-apply-rollback-write");
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();

        // Pre-populate a file that will be overwritten
        fs::create_dir_all(dir.join("packages")).unwrap();
        fs::write(dir.join("packages/keep.nix"), "original").unwrap();

        // Create a proposal that overwrites it and writes a new file
        let proposal = ConfigProposal {
            id: "test-rollback".into(),
            summary: "overwrite test".into(),
            nix_changes: "test".into(),
            files_to_write: vec![
                ("packages/keep.nix".into(), "new".into()),
                ("packages/extra.nix".into(), "extra".into()),
            ],
            files_to_delete: vec![],
            rollback_guidance: "".into(),
        };
        let props_dir = dir.join("proposals");
        fs::create_dir_all(&props_dir).unwrap();
        fs::write(
            props_dir.join("test-rollback.json"),
            serde_json::to_string(&proposal).unwrap(),
        )
        .unwrap();

        // Apply with no-rebuild (simulates what happens before nixos-rebuild)
        let result = execute("test-rollback", false, true, Some(&dir));
        assert!(result.is_ok());

        // Both files should be present with new content
        assert_eq!(
            fs::read_to_string(dir.join("packages/keep.nix")).unwrap(),
            "new"
        );
        assert_eq!(
            fs::read_to_string(dir.join("packages/extra.nix")).unwrap(),
            "extra"
        );

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_snapshot_rollback_deletes() {
        use std::fs;
        let dir = PathBuf::from("/tmp/agntos-apply-rollback-delete");
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        fs::create_dir_all(dir.join("packages")).unwrap();
        fs::write(dir.join("packages/gone.nix"), "will be deleted").unwrap();

        let proposal = ConfigProposal {
            id: "test-delete".into(),
            summary: "delete test".into(),
            nix_changes: "test".into(),
            files_to_write: vec![],
            files_to_delete: vec!["packages/gone.nix".into()],
            rollback_guidance: "".into(),
        };
        let props_dir = dir.join("proposals");
        fs::create_dir_all(&props_dir).unwrap();
        fs::write(
            props_dir.join("test-delete.json"),
            serde_json::to_string(&proposal).unwrap(),
        )
        .unwrap();

        let result = execute("test-delete", false, true, Some(&dir));
        assert!(result.is_ok());
        assert!(!dir.join("packages/gone.nix").exists());

        let _ = fs::remove_dir_all(&dir);
    }
}
