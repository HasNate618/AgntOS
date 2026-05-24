//! Shared utilities for agntd: process execution, user I/O, and file-based inspection fallback.

use crate::log;
use std::io::{self, BufRead, Write};
use std::path::Path;
use std::process::Command;

const AGNTCTL: &str = "agntctl";

pub fn config_dir_str() -> String {
    std::env::var("AGNTOS_CONFIG_DIR").unwrap_or_else(|_| "/etc/agntos".to_string())
}

pub fn find_agntctl() -> String {
    if let Ok(path) = std::env::var("AGNTCTL") {
        if !path.is_empty() && Path::new(&path).is_file() {
            return path;
        }
    }

    for candidate in [
        "/run/current-system/sw/bin/agntctl",
        "/run/wrappers/bin/agntctl",
    ] {
        if Path::new(candidate).is_file() {
            return candidate.to_string();
        }
    }

    if let Ok(output) = Command::new("which").arg(AGNTCTL).output() {
        if output.status.success() {
            let path = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if !path.is_empty() && Path::new(&path).is_file() {
                return path;
            }
        }
    }

    let dev_roots = [
        std::env::var("AGNTOS_SRC").ok(),
        Some("/mnt/agntos-src".to_string()),
        Some(".".to_string()),
        Some("..".to_string()),
    ];
    for root in dev_roots.into_iter().flatten() {
        for sub in ["target/release/agntctl", "target/debug/agntctl"] {
            let p = format!("{}/{}", root.trim_end_matches('/'), sub);
            if Path::new(&p).is_file() {
                return p;
            }
        }
    }

    AGNTCTL.to_string()
}

pub fn log_startup_paths() {
    let agntctl = find_agntctl();
    let exists = Path::new(&agntctl).is_file();
    log::info(&format!(
        "startup config_dir={} agntctl={} exists={} state_dir={}",
        config_dir_str(),
        agntctl,
        exists,
        agnt_common::paths::agent_state_dir().display()
    ));
    log::info(&format!("log file: {}", log::log_path().display()));
    if !exists {
        log::error(&format!(
            "agntctl not found at resolved path — set AGNTCTL or fix systemd Path (see agntd.log)"
        ));
    }
}

pub fn run_agntctl(args: &[&str]) -> Result<(String, String, bool), String> {
    let agntctl_path = find_agntctl();
    let display = Path::new(&agntctl_path)
        .file_name()
        .map(|f| f.to_string_lossy().to_string())
        .unwrap_or_else(|| AGNTCTL.to_string());

    if !Path::new(&agntctl_path).is_file() && agntctl_path == AGNTCTL {
        let msg = format!(
            "Failed to run agntctl: not found on PATH (set AGNTCTL or add agntctl to agntd service Path). Tried: AGNTCTL env, /run/current-system/sw/bin/agntctl, which"
        );
        log::error(&format!("{} args={:?}", msg, args));
        return Err(msg);
    }

    log::info(&format!("exec {} {:?}", agntctl_path, args));

    let output = Command::new(&agntctl_path)
        .args(args)
        .output()
        .map_err(|e| {
            let msg = format!("Failed to run {}: {} (path={})", display, e, agntctl_path);
            log::error(&format!("{} args={:?}", msg, args));
            msg
        })?;

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    let success = output.status.success();

    if success {
        log::info(&format!(
            "ok {} (stdout {} bytes)",
            display,
            stdout.len()
        ));
    } else {
        log::warn(&format!(
            "fail {} exit={:?} stderr={}",
            display,
            output.status.code(),
            stderr.lines().next().unwrap_or("")
        ));
    }

    Ok((stdout, stderr, success))
}

pub fn print_output(stdout: &str, stderr: &str, success: bool) {
    if !stdout.is_empty() {
        for line in stdout.lines() {
            println!("  {}", line);
        }
    }
    if !stderr.is_empty() {
        for line in stderr.lines() {
            eprintln!("  ! {}", line);
            log::warn(&format!("agntctl stderr: {}", line));
        }
    }
    if !success {
        println!("  (command finished with errors)");
    }
}

pub fn confirm(prompt: &str) -> bool {
    print!("{} [y/N] ", prompt);
    io::stdout().flush().unwrap();
    let input = read_line().unwrap_or_default();
    input.trim().to_lowercase() == "y"
}

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

pub fn extract_proposal_id(output: &str) -> Option<String> {
    for line in output.lines() {
        if let Some(stripped) = line.strip_prefix("Proposal: ") {
            return Some(stripped.trim().to_string());
        }
    }
    None
}

pub fn extract_proposal_summary(output: &str) -> Option<String> {
    for line in output.lines() {
        if let Some(stripped) = line.strip_prefix("Summary: ") {
            return Some(stripped.trim().to_string());
        }
    }
    None
}

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
        Err(e) => {
            log::warn(&format!("inspect fallback: {}", e));
            fallback_inspect(target)
        }
    }
}

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
                log::info(&format!("auto_apply applied proposal {}", id));
                eprintln!("  [auto_apply] applied proposal {}", id);
            } else {
                log::warn(&format!("auto_apply failed: {}{}", stderr, stdout));
                eprintln!("  [auto_apply] failed: {}{}", stderr, stdout);
            }
            Ok(())
        }
        Err(e) => Err(e),
    }
}

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

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[test]
    fn find_agntctl_uses_agntctl_env() {
        let fake = format!("/tmp/agntos-test-agntctl-{}", std::process::id());
        std::fs::write(&fake, b"").unwrap();
        env::set_var("AGNTCTL", &fake);
        assert_eq!(find_agntctl(), fake);
        env::remove_var("AGNTCTL");
        let _ = std::fs::remove_file(&fake);
    }
}
