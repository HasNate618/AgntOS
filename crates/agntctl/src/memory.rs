use agnt_common::memory::{CoreMemory, MemoryFile};
use std::path::PathBuf;

const DEFAULT_CONFIG_DIR: &str = "/etc/agntos";

fn load_memory(config_dir: Option<&PathBuf>) -> Result<CoreMemory, String> {
    let dir = config_dir
        .cloned()
        .unwrap_or_else(|| PathBuf::from(DEFAULT_CONFIG_DIR));
    CoreMemory::load(dir)
}

pub fn execute_show(
    file: Option<&str>,
    json: bool,
    config_dir: Option<&PathBuf>,
) -> Result<String, String> {
    let memory = load_memory(config_dir)?;

    if json {
        let payload = match file {
            Some("memory") => serde_json::json!({
                "file": "memory",
                "content": memory.memory,
                "usage_percent": memory.usage_percent(MemoryFile::Memory),
            }),
            Some("user") => serde_json::json!({
                "file": "user",
                "content": memory.user,
                "usage_percent": memory.usage_percent(MemoryFile::User),
            }),
            _ => serde_json::json!({
                "memory": {
                    "content": memory.memory,
                    "usage_percent": memory.usage_percent(MemoryFile::Memory),
                },
                "user": {
                    "content": memory.user,
                    "usage_percent": memory.usage_percent(MemoryFile::User),
                }
            }),
        };
        return serde_json::to_string_pretty(&payload)
            .map_err(|e| format!("Failed to serialize memory output: {}", e));
    }

    let mut out = String::new();
    match file {
        Some("memory") => {
            out.push_str(&format!(
                "MEMORY.md ({}%)\n\n{}",
                memory.usage_percent(MemoryFile::Memory),
                memory.memory
            ));
        }
        Some("user") => {
            out.push_str(&format!(
                "USER.md ({}%)\n\n{}",
                memory.usage_percent(MemoryFile::User),
                memory.user
            ));
        }
        _ => {
            out.push_str(&format!(
                "MEMORY.md ({}%)\n\n{}\n\nUSER.md ({}%)\n\n{}",
                memory.usage_percent(MemoryFile::Memory),
                memory.memory,
                memory.usage_percent(MemoryFile::User),
                memory.user
            ));
        }
    }

    if out.trim().is_empty() {
        Ok("No memory yet. Use `agntctl memory add ...`\n".to_string())
    } else {
        Ok(out)
    }
}

pub fn execute_add(
    file: &str,
    section: &str,
    content: &str,
    config_dir: Option<&PathBuf>,
) -> Result<String, String> {
    let memory_file = MemoryFile::from_str(file)
        .ok_or_else(|| "Invalid file. Use 'memory' or 'user'.".to_string())?;
    let mut core = load_memory(config_dir)?;
    core.add(memory_file, section, content)?;
    let usage = core.usage_percent(memory_file);
    Ok(format!("Added entry to {} ({}% used).\n", file, usage))
}

pub fn execute_replace(
    file: &str,
    target: &str,
    replacement: &str,
    config_dir: Option<&PathBuf>,
) -> Result<String, String> {
    let memory_file = MemoryFile::from_str(file)
        .ok_or_else(|| "Invalid file. Use 'memory' or 'user'.".to_string())?;
    let mut core = load_memory(config_dir)?;
    core.replace(memory_file, target, replacement)?;
    let usage = core.usage_percent(memory_file);
    Ok(format!("Replaced entry in {} ({}% used).\n", file, usage))
}

pub fn execute_remove(
    file: &str,
    target: &str,
    config_dir: Option<&PathBuf>,
) -> Result<String, String> {
    let memory_file = MemoryFile::from_str(file)
        .ok_or_else(|| "Invalid file. Use 'memory' or 'user'.".to_string())?;
    let mut core = load_memory(config_dir)?;
    core.remove(memory_file, target)?;
    let usage = core.usage_percent(memory_file);
    Ok(format!("Removed entry from {} ({}% used).\n", file, usage))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn add_and_show_memory() {
        let dir = std::env::temp_dir().join("agntctl-memory-test");
        let _ = std::fs::remove_dir_all(&dir);

        execute_add("memory", "System", "GPU: QEMU", Some(&dir)).unwrap();
        let out = execute_show(Some("memory"), false, Some(&dir)).unwrap();
        assert!(out.contains("GPU: QEMU"));

        let _ = std::fs::remove_dir_all(&dir);
    }
}
