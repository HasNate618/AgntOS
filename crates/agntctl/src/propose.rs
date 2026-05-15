use agnt_common::config::ConfigProposal;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

const DEFAULT_CONFIG_DIR: &str = "/etc/agntos";

/// Validates a Nix expression by parsing it with `nix-instantiate --parse`.
/// Returns Ok(()) if the expression parses cleanly.
fn validate_nix(content: &str) -> Result<(), String> {
    use std::io::Write;
    use std::sync::atomic::{AtomicU32, Ordering};
    static COUNTER: AtomicU32 = AtomicU32::new(0);
    let mut tmp = std::env::temp_dir();
    tmp.push(format!(
        "agntos-nix-val-{}-{}.nix",
        std::process::id(),
        COUNTER.fetch_add(1, Ordering::Relaxed)
    ));
    let mut f =
        std::fs::File::create(&tmp).map_err(|e| format!("Failed to create temp file: {}", e))?;
    f.write_all(content.as_bytes())
        .map_err(|e| format!("Failed to write temp file: {}", e))?;
    drop(f);

    let output = std::process::Command::new("nix-instantiate")
        .arg("--parse")
        .arg(&tmp)
        .output();

    let _ = std::fs::remove_file(&tmp);

    match output {
        Ok(out) if out.status.success() => Ok(()),
        Ok(out) => {
            let stderr = String::from_utf8_lossy(&out.stderr);
            Err(format!("Nix syntax error:\n{}", stderr.trim()))
        }
        Err(e) => {
            // nix-instantiate not available — skip validation
            if e.kind() == std::io::ErrorKind::NotFound {
                Ok(())
            } else {
                Err(format!("nix-instantiate failed: {}", e))
            }
        }
    }
}

pub fn execute(
    description: &str,
    dry_run: bool,
    config_dir: Option<&PathBuf>,
) -> Result<String, String> {
    let dir = config_dir
        .cloned()
        .unwrap_or_else(|| PathBuf::from(DEFAULT_CONFIG_DIR));

    let proposal = generate(description)?;

    if !dry_run {
        let proposals_dir = dir.join("proposals");
        std::fs::create_dir_all(&proposals_dir)
            .map_err(|e| format!("Failed to create proposals dir: {}", e))?;

        let proposal_path = proposals_dir.join(format!("{}.json", proposal.id));
        let json = serde_json::to_string_pretty(&proposal)
            .map_err(|e| format!("Failed to serialize proposal: {}", e))?;
        std::fs::write(&proposal_path, &json)
            .map_err(|e| format!("Failed to write proposal: {}", e))?;
    }

    let out = format!(
        "Proposal: {}\n\
         Summary:  {}\n\
         \n\
         Nix changes:\n\
         {}\n\
         \n\
         Rollback: {}\n",
        proposal.id, proposal.summary, proposal.nix_changes, proposal.rollback_guidance,
    );

    if !dry_run {
        let out = format!(
            "{}\n\
             Staged at: {}/proposals/{}.json\n\
             Run `agntctl apply {}` to apply.\n",
            out.trim(),
            dir.display(),
            proposal.id,
            proposal.id,
        );
        Ok(out)
    } else {
        Ok(format!("{}\nDRY RUN — not saved.\n", out.trim()))
    }
}

fn generate(description: &str) -> Result<ConfigProposal, String> {
    let proposal = generate_raw(description)?;
    for (_, content) in &proposal.files_to_write {
        validate_nix(content)?;
    }
    Ok(proposal)
}

fn generate_raw(description: &str) -> Result<ConfigProposal, String> {
    let lower = description.trim().to_lowercase();
    let id = generate_id();

    if lower.starts_with("install ") {
        let rest = lower.strip_prefix("install ").unwrap().trim();
        let package = sanitize_package_name(rest);
        let file_path = format!("packages/{}.nix", package);
        Ok(ConfigProposal {
            id,
            summary: format!("Install package: {}", package),
            nix_changes: format!("Add pkgs.{} to environment.systemPackages", package),
            files_to_write: vec![(
                file_path.clone(),
                format!(
                    "{{ config, lib, pkgs, ... }}: {{\n  environment.systemPackages = lib.mkAfter [ pkgs.{} ];\n}}\n",
                    package
                ),
            )],
            files_to_delete: vec![],
            rollback_guidance: format!(
                "To rollback: delete {} or run `agntctl propose remove {}`.",
                file_path, package
            ),
        })
    } else if lower.starts_with("remove ") || lower.starts_with("uninstall ") {
        let prefix = if lower.starts_with("remove ") {
            "remove "
        } else {
            "uninstall "
        };
        let package = sanitize_package_name(lower.strip_prefix(prefix).unwrap().trim());
        let file_path = format!("packages/{}.nix", package);
        Ok(ConfigProposal {
            id,
            summary: format!("Remove package: {}", package),
            nix_changes: format!(
                "Delete {} (removes pkgs.{} from environment.systemPackages)",
                file_path, package
            ),
            files_to_write: vec![],
            files_to_delete: vec![file_path.clone()],
            rollback_guidance: format!(
                "To rollback: run `agntctl propose install {}` to re-create the module.",
                package
            ),
        })
    } else if lower.starts_with("enable ") {
        let service = sanitize_package_name(lower.strip_prefix("enable ").unwrap().trim());
        let services_path = to_services_path(&service);
        Ok(ConfigProposal {
            id,
            summary: format!("Enable service: {}", service),
            nix_changes: format!(
                "services.{} = {{\n  enable = true;\n}};",
                services_path
            ),
            files_to_write: vec![(
                format!("services/{}.nix", service),
                format!(
                    "{{ config, pkgs, ... }}: {{\n  services.{} = {{\n    enable = true;\n  }};\n}}",
                    services_path
                ),
            )],
            files_to_delete: vec![],
            rollback_guidance: format!(
                "To rollback: set `services.{}.enable = false`, then run `nixos-rebuild switch`.",
                services_path
            ),
        })
    } else if lower.starts_with("disable ") {
        let service = sanitize_package_name(lower.strip_prefix("disable ").unwrap().trim());
        let file_path = format!("services/{}.nix", service);
        Ok(ConfigProposal {
            id,
            summary: format!("Disable service: {}", service),
            nix_changes: format!(
                "Delete {} (service falls back to disabled default)",
                file_path
            ),
            files_to_write: vec![],
            files_to_delete: vec![file_path.clone()],
            rollback_guidance: format!(
                "To rollback: run `agntctl propose enable {}` to re-create the service module.",
                service
            ),
        })
    } else if lower.starts_with("set ") {
        let rest = description
            .trim()
            .strip_prefix("set ")
            .or_else(|| description.trim().strip_prefix("SET "))
            .unwrap_or("");
        let (option_path, value) = if let Some(idx) = rest.find(" = ") {
            (
                rest[..idx].trim().to_string(),
                rest[idx + 3..].trim().to_string(),
            )
        } else if let Some(idx) = rest.find(' ') {
            (
                rest[..idx].trim().to_string(),
                rest[idx + 1..].trim().to_string(),
            )
        } else {
            (rest.trim().to_string(), "true".to_string())
        };
        let value_expr = if value == "true" || value == "false" {
            value
        } else if value.starts_with('[') || value.starts_with('{') {
            value
        } else if value.parse::<f64>().is_ok() {
            value
        } else {
            format!("\"{}\"", value.trim_matches('"'))
        };
        let file_path = format!("options/{}.nix", option_path.replace('.', "-"));
        Ok(ConfigProposal {
            id,
            summary: format!("Set option: {} = {}", option_path, value_expr),
            nix_changes: format!("{}.{} = {};", option_path, option_path, value_expr),
            files_to_write: vec![(
                file_path,
                format!(
                    "{{ config, lib, pkgs, ... }}: {{\n  {}.{} = {};\n}}\n",
                    option_path, option_path, value_expr
                ),
            )],
            files_to_delete: vec![],
            rollback_guidance: format!(
                "To rollback: run `agntctl propose set {} <original-value>`.",
                option_path
            ),
        })
    } else {
        Ok(ConfigProposal {
            id,
            summary: format!("Custom change: {}", description),
            nix_changes: format!(
                "# TODO: {}\n#\n# This is a custom proposal that needs refinement.\n",
                description
            ),
            files_to_write: vec![(
                "custom.nix".to_string(),
                format!(
                    "{{ config, pkgs, ... }}: {{\n  # TODO: {}\n}}\n",
                    description
                ),
            )],
            files_to_delete: vec![],
            rollback_guidance: "Check the audit log before rolling back custom changes."
                .to_string(),
        })
    }
}

fn generate_id() -> String {
    let ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    format!("p-{:x}", ts)
}

fn sanitize_package_name(name: &str) -> String {
    name.trim()
        .to_lowercase()
        .replace(' ', "-")
        .replace(|c: char| !c.is_alphanumeric() && c != '-', "")
}

fn to_services_path(name: &str) -> String {
    name.replace('-', ".")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_install() {
        let p = generate("install firefox").unwrap();
        assert!(p.summary.contains("firefox"));
        assert!(p.nix_changes.contains("pkgs.firefox"));
        assert_eq!(p.files_to_write[0].0, "packages/firefox.nix");
        assert!(p.files_to_write[0].1.contains("lib.mkAfter"));
        assert!(p.files_to_delete.is_empty());
    }

    #[test]
    fn test_generate_remove() {
        let p = generate("remove firefox").unwrap();
        assert!(p.summary.contains("firefox"));
        assert!(p.nix_changes.contains("Delete"));
        assert!(p.files_to_write.is_empty());
        assert_eq!(p.files_to_delete, vec!["packages/firefox.nix"]);
    }

    #[test]
    fn test_generate_enable() {
        let p = generate("enable docker").unwrap();
        assert!(p.summary.contains("docker"));
        assert!(p.nix_changes.contains("enable = true"));
    }

    #[test]
    fn test_generate_disable() {
        let p = generate("disable sshd").unwrap();
        assert!(p.summary.contains("sshd"));
        assert!(p.nix_changes.contains("Delete services/sshd.nix"));
        assert!(p.files_to_write.is_empty());
        assert_eq!(p.files_to_delete, vec!["services/sshd.nix"]);
    }

    #[test]
    fn test_generic_fallback() {
        let p = generate("some custom thing").unwrap();
        assert!(p.summary.contains("some custom thing"));
        assert!(p.nix_changes.contains("TODO"));
    }

    #[test]
    fn test_generate_set_string() {
        let p = generate("set networking.hostName myhost").unwrap();
        assert!(p.summary.contains("networking.hostName"));
        assert!(p.files_to_write[0].1.contains("\"myhost\""));
    }

    #[test]
    fn test_generate_set_bool() {
        let p = generate("set services.openssh.enable true").unwrap();
        assert!(p.files_to_write[0].1.contains("true"));
    }

    #[test]
    fn test_generate_set_int() {
        let p = generate("set boot.kernel.sysctl.vm.swappiness 10").unwrap();
        assert!(p.files_to_write[0].1.contains("= 10;"));
    }

    #[test]
    fn test_sanitize_package() {
        assert_eq!(sanitize_package_name("VS Code"), "vs-code");
        assert_eq!(sanitize_package_name("Firefox"), "firefox");
        assert_eq!(sanitize_package_name("  Docker "), "docker");
    }

    #[test]
    fn test_execute_dry_run() {
        let result = execute("install hello", true, None).unwrap();
        assert!(result.contains("DRY RUN"));
        assert!(result.contains("hello"));
    }

    #[test]
    fn test_execute_writes_file() {
        use std::fs;
        let dir = std::env::temp_dir().join(format!("agntos-test-{}", generate_id()));
        let result = execute("install hello", false, Some(&dir)).unwrap();
        assert!(!result.contains("DRY RUN"));
        assert!(result.contains("Staged at"));

        // Verify the file was written
        let props = dir.join("proposals");
        assert!(props.exists());
        let entries: Vec<_> = fs::read_dir(&props).unwrap().collect();
        assert!(entries.len() > 0);

        // Cleanup
        let _ = fs::remove_dir_all(&dir);
    }
}
