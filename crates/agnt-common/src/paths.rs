use std::path::{Path, PathBuf};

pub fn agent_state_dir() -> PathBuf {
    if let Ok(dir) = std::env::var("AGNTOS_STATE_DIR") {
        if !dir.is_empty() {
            return PathBuf::from(dir);
        }
    }
    if let Ok(xdg) = std::env::var("XDG_STATE_HOME") {
        if !xdg.is_empty() {
            return PathBuf::from(xdg).join("agntos");
        }
    }
    dirs_home_state().join("agntos")
}

pub fn nix_config_dir() -> PathBuf {
    std::env::var("AGNTOS_CONFIG_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("/etc/agntos"))
}

pub fn memory_dir() -> PathBuf {
    agent_state_dir().join("memory")
}

fn dirs_home_state() -> PathBuf {
    std::env::var("HOME")
        .map(|h| PathBuf::from(h).join(".local/state"))
        .unwrap_or_else(|_| PathBuf::from("/var/lib/agntos-state"))
}

pub fn migrate_memory_from_config(config_dir: impl AsRef<Path>) -> Result<(), String> {
    let legacy = config_dir.as_ref().join("memory");
    let target = memory_dir();
    if !legacy.is_dir() {
        return Ok(());
    }
    let has_state = target.join("MEMORY.md").exists() || target.join("USER.md").exists();
    if has_state {
        return Ok(());
    }
    std::fs::create_dir_all(&target).map_err(|e| e.to_string())?;
    for name in ["MEMORY.md", "USER.md"] {
        let src = legacy.join(name);
        if src.exists() {
            std::fs::copy(&src, target.join(name)).map_err(|e| e.to_string())?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn agent_state_dir_respects_override() {
        std::env::set_var("AGNTOS_STATE_DIR", "/tmp/agntos-test-state");
        assert_eq!(agent_state_dir(), PathBuf::from("/tmp/agntos-test-state"));
        std::env::remove_var("AGNTOS_STATE_DIR");
    }
}
