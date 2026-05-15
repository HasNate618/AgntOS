use agnt_common::config::ConfigProposal;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

const DEFAULT_CONFIG_DIR: &str = "/etc/agntos";

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
        let services_path = to_services_path(&service);
        Ok(ConfigProposal {
            id,
            summary: format!("Disable service: {}", service),
            nix_changes: format!(
                "services.{} = {{\n  enable = false;\n}};",
                services_path
            ),
            files_to_write: vec![(
                format!("services/{}.nix", service),
                format!(
                    "{{ config, pkgs, ... }}: {{\n  services.{} = {{\n    enable = false;\n  }};\n}}",
                    services_path
                ),
            )],
            files_to_delete: vec![],
            rollback_guidance: format!(
                "To rollback: set `services.{}.enable = true`, then run `nixos-rebuild switch`.",
                services_path
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
        assert!(p.nix_changes.contains("enable = false"));
    }

    #[test]
    fn test_generic_fallback() {
        let p = generate("some custom thing").unwrap();
        assert!(p.summary.contains("some custom thing"));
        assert!(p.nix_changes.contains("TODO"));
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
