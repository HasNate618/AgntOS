// agntd - AgntOS agent daemon.
//
// CLI chat loop that parses user intent, calls agntctl tools,
// shows output, and handles approval flows.

use std::io::{self, BufRead, Write};
use std::process::Command;

const AGNTCTL: &str = "agntctl";

fn config_dir_str() -> String {
    std::env::var("AGNTOS_CONFIG_DIR").unwrap_or_else(|_| "/etc/agntos".to_string())
}

fn find_agntctl() -> String {
    // Check PATH first
    if let Ok(output) = std::process::Command::new("which").arg(AGNTCTL).output() {
        if output.status.success() {
            let path = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if !path.is_empty() {
                return path;
            }
        }
    }

    // Dev paths relative to repository root
    let dev_paths = [
        "target/debug/agntctl",
        "target/release/agntctl",
        "../target/debug/agntctl",
        "../target/release/agntctl",
    ];

    for p in &dev_paths {
        if std::path::Path::new(p).exists() {
            return p.to_string();
        }
    }

    AGNTCTL.to_string()
}

fn main() {
    println!("agntd: AgntOS agent daemon");
    println!("type 'help' for commands, 'quit' to exit.\n");

    let mut history: Vec<String> = Vec::new();

    loop {
        print!("agnt> ");
        io::stdout().flush().unwrap();

        let input = read_line();
        let input = match input {
            Some(line) => line.trim().to_string(),
            None => {
                println!();
                break;
            }
        };

        if input.is_empty() {
            continue;
        }

        history.push(input.clone());

        let lower = input.to_lowercase();

        match lower.as_str() {
            "quit" | "exit" | "bye" => {
                println!("bye.");
                break;
            }
            "help" | "?" => {
                show_help();
            }
            _ if lower.starts_with("inspect ") || lower == "inspect" => {
                handle_inspect(&lower, &input);
            }
            _ if lower.starts_with("install ") => {
                handle_install(&lower, &input);
            }
            _ if lower.starts_with("remove ") || lower.starts_with("uninstall ") => {
                handle_remove(&lower, &input);
            }
            _ if lower.starts_with("enable ") => {
                handle_enable(&lower, &input);
            }
            _ if lower.starts_with("disable ") => {
                handle_disable(&lower, &input);
            }
            _ if lower.starts_with("propose ") => {
                handle_propose(&lower, &input);
            }
            _ if lower.starts_with("apply ") => {
                handle_apply(&lower);
            }
            _ if lower == "audit" || lower.starts_with("audit ") => {
                handle_audit(&lower);
            }
            _ => {
                // Fallback: pass to agntctl as generic propose
                println!("  I didn't understand that. Trying as a generic proposal...");
                handle_propose(&lower, &input);
            }
        }
    }
}

fn read_line() -> Option<String> {
    let stdin = io::stdin();
    let mut handle = stdin.lock();
    let mut line = String::new();
    match handle.read_line(&mut line) {
        Ok(0) => None,
        Ok(_) => Some(line),
        Err(_) => None,
    }
}

fn show_help() {
    println!("  Commands:");
    println!("  inspect [target]       Inspect system (system, cpu, memory, gpu, disk, network)");
    println!("  install <package>      Propose installing a package");
    println!("  remove <package>       Propose removing a package");
    println!("  enable <service>       Propose enabling a service");
    println!("  disable <service>      Propose disabling a service");
    println!("  propose <description>  Propose a custom Nix config change");
    println!("  apply <id>             Apply an approved proposal");
    println!("  audit                  Show recent audit log");
    println!("  audit show <id>        Show audit entry details");
    println!("  help                   Show this help");
    println!("  quit | exit            Exit");
}

fn run_agntctl(args: &[&str]) -> Result<(String, String, bool), String> {
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

fn print_output(stdout: &str, stderr: &str, success: bool) {
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

fn confirm(prompt: &str) -> bool {
    print!("{} [y/N] ", prompt);
    io::stdout().flush().unwrap();
    let input = read_line().unwrap_or_default();
    input.trim().to_lowercase() == "y"
}

fn handle_inspect(lower: &str, _original: &str) {
    let target = lower.strip_prefix("inspect ").unwrap_or("system");
    let target = target.split_whitespace().next().unwrap_or("system");

    let stdout = capture_inspect(target);

    let msg = format!("Inspects the running system for hardware and OS information.");
    println!("  {}", msg);
    println!();

    if !stdout.is_empty() {
        println!("{}", stdout);
    }

    println!("  (use `audit` to view the activity log for details)");
}

fn capture_inspect(target: &str) -> String {
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
                format!("(inspect failed)")
            }
        }
        Err(_e) => {
            fallback_inspect(target)
        }
    }
}

fn fallback_inspect(target: &str) -> String {
    // Minimal fallback when agntctl is not available
    let hostname = std::process::Command::new("hostname")
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .unwrap_or_else(|_| "unknown".into());

    match target {
        "cpu" => {
            let model = std::fs::read_to_string("/proc/cpuinfo")
                .ok()
                .and_then(|s| s.lines().find(|l| l.starts_with("model name"))
                    .map(|l| l.split(':').nth(1).unwrap_or("unknown").trim().to_string()))
                .unwrap_or_else(|| "unknown".into());
            let cores = std::fs::read_to_string("/proc/cpuinfo")
                .ok()
                .map(|s| s.lines().filter(|l| l.trim().starts_with("processor")).count())
                .unwrap_or(0);
            format!("CPU: {}\nCores: {}", model, cores)
        }
        "memory" | "mem" => {
            std::fs::read_to_string("/proc/meminfo")
                .ok()
                .and_then(|s| {
                    s.lines().find(|l| l.starts_with("MemTotal"))
                        .map(|l| format!("Memory Total: {}", l.split(':').nth(1).unwrap_or("unknown").trim()))
                })
                .unwrap_or_else(|| "Memory: unknown".into())
        }
        _ => {
            let os = std::fs::read_to_string("/etc/os-release")
                .ok()
                .and_then(|s| s.lines().find(|l| l.starts_with("PRETTY_NAME"))
                    .map(|l| l.split('=').nth(1).unwrap_or("unknown").trim_matches('"').to_string()))
                .unwrap_or_else(|| "AgntOS (unknown)".into());
            let kernel = std::process::Command::new("uname")
                .arg("-r")
                .output()
                .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
                .unwrap_or_else(|_| "unknown".into());
            format!("{}\nHostname: {}\nKernel: {}", os, hostname, kernel)
        }
    }
}

fn handle_install(lower: &str, _original: &str) {
    let package = lower.strip_prefix("install ").unwrap().trim();
    if package.is_empty() {
        println!("  Usage: install <package>");
        return;
    }

    println!("  Proposing installation of: {}", package);
    let propose_output = propose_change("install", package);

    if propose_output.is_empty() {
        return;
    }

    // Show proposal
    println!("{}", propose_output);

    // Offer to apply
    if confirm("\n  Apply this proposal?") {
        // Extract proposal ID from output
        if let Some(id) = extract_proposal_id(&propose_output) {
            apply_proposal(&id);
        } else {
            println!("  Could not find proposal ID.");
        }
    }
}

fn handle_remove(lower: &str, _original: &str) {
    let prefix = if lower.starts_with("remove ") { "remove " } else { "uninstall " };
    let package = lower.strip_prefix(prefix).unwrap().trim();
    if package.is_empty() {
        println!("  Usage: remove <package>");
        return;
    }

    println!("  Proposing removal of: {}", package);
    let propose_output = propose_change("remove", package);

    if propose_output.is_empty() {
        return;
    }

    println!("{}", propose_output);

    if confirm("\n  Apply this proposal?") {
        if let Some(id) = extract_proposal_id(&propose_output) {
            apply_proposal(&id);
        }
    }
}

fn handle_enable(lower: &str, _original: &str) {
    let service = lower.strip_prefix("enable ").unwrap().trim();
    if service.is_empty() {
        println!("  Usage: enable <service>");
        return;
    }

    println!("  Proposing enable of service: {}", service);
    let propose_output = propose_change("enable", service);

    if propose_output.is_empty() {
        return;
    }

    println!("{}", propose_output);

    if confirm("\n  Apply this proposal?") {
        if let Some(id) = extract_proposal_id(&propose_output) {
            apply_proposal(&id);
        }
    }
}

fn handle_disable(lower: &str, _original: &str) {
    let service = lower.strip_prefix("disable ").unwrap().trim();
    if service.is_empty() {
        println!("  Usage: disable <service>");
        return;
    }

    println!("  Proposing disable of service: {}", service);
    let propose_output = propose_change("disable", service);

    if propose_output.is_empty() {
        return;
    }

    println!("{}", propose_output);

    if confirm("\n  Apply this proposal?") {
        if let Some(id) = extract_proposal_id(&propose_output) {
            apply_proposal(&id);
        }
    }
}

fn propose_change(verb: &str, target: &str) -> String {
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

fn handle_propose(lower: &str, _original: &str) {
    let description = lower.strip_prefix("propose ").unwrap_or(lower);
    if description.is_empty() {
        println!("  Usage: propose <description>");
        return;
    }

    let output = propose_change("propose", description);
    if output.is_empty() {
        return;
    }

    println!("{}", output);

    if confirm("\n  Apply this proposal?") {
        if let Some(id) = extract_proposal_id(&output) {
            apply_proposal(&id);
        }
    }
}

fn handle_apply(lower: &str) {
    let id = lower.strip_prefix("apply ").unwrap_or(lower).split_whitespace().next().unwrap_or("");
    if id.is_empty() {
        println!("  Usage: apply <proposal-id>");
        return;
    }

    if !confirm(&format!("  Apply proposal {}? This may change system configuration.", id)) {
        println!("  Cancelled.");
        return;
    }

    apply_proposal(id);
}

fn apply_proposal(id: &str) {
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

fn handle_audit(lower: &str) {
    let parts: Vec<&str> = lower.split_whitespace().collect();

    match parts.as_slice() {
        ["audit"] | ["audit", "list"] => {
            let cfg = config_dir_str();
            match run_agntctl(&["audit", "list", "--config-dir", &cfg]) {
                Ok((stdout, stderr, success)) => print_output(&stdout, &stderr, success),
                Err(e) => println!("  Error: {}", e),
            }
        }
        ["audit", "show", id] => {
            let cfg = config_dir_str();
            match run_agntctl(&["audit", "show", id, "--config-dir", &cfg]) {
                Ok((stdout, stderr, success)) => print_output(&stdout, &stderr, success),
                Err(e) => println!("  Error: {}", e),
            }
        }
        _ => {
            println!("  Usage: audit [list|show <id>]");
        }
    }
}

fn extract_proposal_id(output: &str) -> Option<String> {
    for line in output.lines() {
        if let Some(stripped) = line.strip_prefix("Proposal: ") {
            return Some(stripped.trim().to_string());
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_proposal_id() {
        let output = "Proposal: p-abc123\nSummary: Install firefox";
        assert_eq!(extract_proposal_id(output), Some("p-abc123".to_string()));
    }

    #[test]
    fn test_extract_proposal_id_no_match() {
        assert_eq!(extract_proposal_id("No proposal here"), None);
    }

    #[test]
    fn test_config_dir_str_default() {
        let dir = config_dir_str();
        assert!(!dir.is_empty(), "config dir should not be empty");
    }

    #[test]
    fn test_find_agntctl_returns_something() {
        let path = find_agntctl();
        assert!(!path.is_empty(), "find_agntctl should return a non-empty path");
    }
}
