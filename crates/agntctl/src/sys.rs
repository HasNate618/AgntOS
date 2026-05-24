//! `agntctl` general-purpose tools — Pi-inspired primitives (read, write, edit, bash).
//!
//! These four tools replace dozens of specialized commands. The agent uses `bash`
//! for `ls`, `grep`, `find`, `systemctl`, `journalctl`, `dmesg`, and anything
//! without a dedicated tool.

use std::path::PathBuf;

/// Reads a file and returns its content as a string.
pub fn execute_read(path: &str) -> Result<String, String> {
    std::fs::read_to_string(path).map_err(|e| format!("Failed to read {}: {}", path, e))
}

/// Creates or overwrites a file with the given content.  Logs the write to the
/// audit log.
pub fn execute_write(
    path: &str,
    content: &str,
    config_dir: Option<&PathBuf>,
) -> Result<String, String> {
    std::fs::write(path, content).map_err(|e| format!("Failed to write {}: {}", path, e))?;

    crate::audit::log_write(path, content.len(), config_dir);

    Ok(format!("Wrote {} ({} bytes).\n", path, content.len()))
}

/// Reads a file, replaces the first occurrence of `old_string` with `new_string`,
/// and writes it back.  Logs the edit to the audit log.
pub fn execute_edit(
    path: &str,
    old_string: &str,
    new_string: &str,
    config_dir: Option<&PathBuf>,
) -> Result<String, String> {
    let original =
        std::fs::read_to_string(path).map_err(|e| format!("Failed to read {}: {}", path, e))?;

    if !original.contains(old_string) {
        return Err(format!("String '{}' not found in {}", old_string, path));
    }

    let edited = original.replacen(old_string, new_string, 1);
    std::fs::write(path, &edited).map_err(|e| format!("Failed to write {}: {}", path, e))?;

    crate::audit::log_edit(path, old_string, new_string, config_dir);

    Ok(format!(
        "Edited {}: replaced '{}' with '{}'.\n",
        path, old_string, new_string
    ))
}

/// Runs `bash -c <command>`, captures stdout and stderr, and optionally logs
/// the execution to the audit log.
pub fn execute_bash(
    command: &str,
    config_dir: Option<&PathBuf>,
    no_audit: bool,
) -> Result<String, String> {
    let output = std::process::Command::new("bash")
        .arg("-c")
        .arg(command)
        .output()
        .map_err(|e| format!("Failed to execute bash: {}", e))?;

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    let exit_code = output.status.code().unwrap_or(-1);

    if !no_audit {
        crate::audit::log_bash(command, exit_code, &stdout, &stderr, config_dir);
    }

    let mut result = String::new();
    if !stdout.is_empty() {
        result.push_str(&stdout);
    }
    if !stderr.is_empty() {
        if !result.is_empty() {
            result.push('\n');
        }
        result.push_str("STDERR:\n");
        result.push_str(&stderr);
    }
    if !output.status.success() {
        if !result.is_empty() {
            result.push('\n');
        }
        result.push_str(&format!("[exit {}]", exit_code));
    }

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_read_file() {
        let dir = std::env::temp_dir().join("agntctl-sys-read-test");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("test.txt");
        std::fs::write(&path, "hello world\n").unwrap();

        let result = execute_read(&path.display().to_string()).unwrap();
        assert_eq!(result, "hello world\n");

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_read_missing_file() {
        let result = execute_read("/nonexistent/file/path.xyz");
        assert!(result.is_err());
    }

    #[test]
    fn test_write_file() {
        let dir = std::env::temp_dir().join("agntctl-sys-write-test");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("out.txt");

        let result = execute_write(&path.display().to_string(), "content here", None).unwrap();
        assert!(result.contains("bytes"));

        let written = std::fs::read_to_string(&path).unwrap();
        assert_eq!(written, "content here");

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_edit_file() {
        let dir = std::env::temp_dir().join("agntctl-sys-edit-test");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("config.txt");
        std::fs::write(&path, "port = 8080\nhost = localhost\n").unwrap();

        let result = execute_edit(
            &path.display().to_string(),
            "port = 8080",
            "port = 9090",
            None,
        )
        .unwrap();
        assert!(result.contains("Edited"));

        let edited = std::fs::read_to_string(&path).unwrap();
        assert!(edited.contains("port = 9090"));
        assert!(edited.contains("host = localhost"));

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_edit_not_found() {
        let dir = std::env::temp_dir().join("agntctl-sys-edit-miss");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("f.txt");
        std::fs::write(&path, "abc").unwrap();

        let result = execute_edit(&path.display().to_string(), "xyz", "123", None);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("not found"));

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_bash_echo() {
        let result = execute_bash("echo hello", None, false).unwrap();
        assert!(result.contains("hello"));
    }

    #[test]
    fn test_bash_nonzero_exit_returns_output() {
        let result = execute_bash("exit 1", None, false).unwrap();
        assert!(result.contains("[exit 1]"));
    }
}
