//! Shared utilities for agntd: process execution, user I/O, and file-based inspection fallback.

use std::io::{self, BufRead, Write};
use std::process::Command;

/// The name of the control-tool binary that agntd delegates to.
const AGNTCTL: &str = "agntctl";

/// Returns the AgntOS config directory from the environment variable `AGNTOS_CONFIG_DIR`,
/// defaulting to `/etc/agntos`.
pub fn config_dir_str() -> String {
    std::env::var("AGNTOS_CONFIG_DIR").unwrap_or_else(|_| "/etc/agntos".to_string())
}

/// Locates the `agntctl` binary.  Tries `PATH` first, then common dev-build paths
/// relative to the repository root (preferring release over debug).
pub fn find_agntctl() -> String {
    if let Ok(output) = Command::new("which").arg(AGNTCTL).output() {
        if output.status.success() {
            let path = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if !path.is_empty() {
                return path;
            }
        }
    }

    let dev_paths = [
        "target/release/agntctl",
        "../target/release/agntctl",
        "target/debug/agntctl",
        "../target/debug/agntctl",
    ];

    for p in &dev_paths {
        if std::path::Path::new(p).exists() {
            return p.to_string();
        }
    }

    AGNTCTL.to_string()
}

/// Runs `agntctl` with the given arguments and returns (stdout, stderr, success).
/// Arguments are passed directly — no shell interpolation.
pub fn run_agntctl(args: &[&str]) -> Result<(String, String, bool), String> {
    let agntctl_path = find_agntctl();
    let display = std::path::Path::new(&agntctl_path)
        .file_name()
        .map(|f| f.to_string_lossy())
        .unwrap_or(std::borrow::Cow::Borrowed(AGNTCTL));

    let output = Command::new(&agntctl_path)
        .args(args)
        .output()
        .map_err(|e| format!("Failed to run {}: {} (is it built?)", display, e))?;

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    let success = output.status.success();

    Ok((stdout, stderr, success))
}

/// Prints stdout lines indented and stderr lines with a warning prefix.
pub fn print_output(stdout: &str, stderr: &str, success: bool) {
    if !stdout.is_empty() {
        for line in stdout.lines() {
            println!("  {}", line);
        }
    }
    if !stderr.is_empty() {
        for line in stderr.lines() {
            eprintln!("  ! {}", line);
        }
    }
    if !success {
        println!("  (command finished with errors)");
    }
}

/// Interactive yes/no confirmation prompt.  Returns `true` when the user answers `y`.
pub fn confirm(prompt: &str) -> bool {
    print!("{} [y/N] ", prompt);
    io::stdout().flush().unwrap();
    let input = read_line().unwrap_or_default();
    input.trim().to_lowercase() == "y"
}

/// Reads a single line from stdin.  Returns `None` on EOF.
pub fn read_line() -> Option<String> {
    let stdin = io::stdin();
    let mut handle = stdin.lock();
    let mut line = String::new();
    match handle.read_line(&mut line) {
        Ok(0) => None,
        Ok(_) => Some(line),
        Err(_) => None,
    }
}

/// Extracts a proposal ID from `agntctl propose` output (looks for `"Proposal: <id>"`).
pub fn extract_proposal_id(output: &str) -> Option<String> {
    for line in output.lines() {
        if let Some(stripped) = line.strip_prefix("Proposal: ") {
            return Some(stripped.trim().to_string());
        }
    }
    None
}

/// Calls `agntctl inspect <target>` and returns its stdout as a plain string.
/// Falls back to `/proc`-based inspection when `agntctl` is unavailable.
pub fn capture_inspect(target: &str) -> String {
    match run_agntctl(&["inspect", target]) {
        Ok((stdout, stderr, success)) => {
            if !stderr.is_empty() {
                for line in stderr.lines() {
                    eprintln!("  ! {}", line);
                }
            }
            if success {
                stdout
            } else {
                "(inspect failed)".to_string()
            }
        }
        Err(_e) => fallback_inspect(target),
    }
}

/// Direct `/proc` and `/etc/os-release` inspection used when `agntctl` is not available.
pub fn fallback_inspect(target: &str) -> String {
    let hostname = Command::new("hostname")
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .unwrap_or_else(|_| "unknown".into());

    match target {
        "cpu" => {
            let model = std::fs::read_to_string("/proc/cpuinfo")
                .ok()
                .and_then(|s| {
                    s.lines()
                        .find(|l| l.starts_with("model name"))
                        .map(|l| l.split(':').nth(1).unwrap_or("unknown").trim().to_string())
                })
                .unwrap_or_else(|| "unknown".into());
            let cores = std::fs::read_to_string("/proc/cpuinfo")
                .ok()
                .map(|s| {
                    s.lines()
                        .filter(|l| l.trim().starts_with("processor"))
                        .count()
                })
                .unwrap_or(0);
            format!("CPU: {}\nCores: {}", model, cores)
        }
        "memory" | "mem" => std::fs::read_to_string("/proc/meminfo")
            .ok()
            .and_then(|s| {
                s.lines().find(|l| l.starts_with("MemTotal")).map(|l| {
                    format!(
                        "Memory Total: {}",
                        l.split(':').nth(1).unwrap_or("unknown").trim()
                    )
                })
            })
            .unwrap_or_else(|| "Memory: unknown".into()),
        _ => {
            let os = std::fs::read_to_string("/etc/os-release")
                .ok()
                .and_then(|s| {
                    s.lines().find(|l| l.starts_with("PRETTY_NAME")).map(|l| {
                        l.split('=')
                            .nth(1)
                            .unwrap_or("unknown")
                            .trim_matches('"')
                            .to_string()
                    })
                })
                .unwrap_or_else(|| "AgntOS (unknown)".into());
            let kernel = Command::new("uname")
                .arg("-r")
                .output()
                .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
                .unwrap_or_else(|_| "unknown".into());
            format!("{}\nHostname: {}\nKernel: {}", os, hostname, kernel)
        }
    }
}

/// Shortcut for `"propose <verb> <target>"` via `agntctl`.
pub fn propose_change(verb: &str, target: &str) -> String {
    let description = format!("{} {}", verb, target);
    let cfg = config_dir_str();
    match run_agntctl(&["propose", "--config-dir", &cfg, &description]) {
        Ok((stdout, stderr, _success)) => {
            if !stderr.is_empty() {
                for line in stderr.lines() {
                    eprintln!("  ! {}", line);
                }
            }
            stdout
        }
        Err(e) => {
            eprintln!("  Error: {}", e);
            String::new()
        }
    }
}

pub fn maybe_auto_apply_after_propose(config_dir: &str, propose_output: &str) -> Result<(), String> {
    use agnt_common::settings::{AgntosSettings, ApplyPolicy};
    if AgntosSettings::load_from_config_dir(config_dir).auto_apply != ApplyPolicy::Auto {
        return Ok(());
    }
    let id = extract_proposal_id(propose_output)
        .ok_or_else(|| "auto_apply: could not parse proposal id from propose output".to_string())?;
    match run_agntctl(&["apply", "--config-dir", config_dir, &id]) {
        Ok((stdout, stderr, success)) => {
            if success {
                eprintln!("  [auto_apply] applied proposal {}", id);
            } else {
                eprintln!("  [auto_apply] failed: {}{}", stderr, stdout);
            }
            Ok(())
        }
        Err(e) => Err(e),
    }
}

/// Shortcut for applying a proposal by ID via `agntctl`.
pub fn apply_proposal(id: &str) {
    let cfg = config_dir_str();
    match run_agntctl(&["apply", "--config-dir", &cfg, id]) {
        Ok((stdout, stderr, success)) => {
            print_output(&stdout, &stderr, success);
            if success {
                println!("  Applied. Run `audit` to see the entry.");
            }
        }
        Err(e) => {
            println!("  Error: {}", e);
        }
    }
}
