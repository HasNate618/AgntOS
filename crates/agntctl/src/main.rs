// agntctl - AgntOS control tool for Nix-backed system changes.
//
// Commands:
//   inspect  - Read-only system inspection
//   propose  - Plan a config change without applying
//   apply    - Apply an approved proposal
//   audit    - View activity log
//   rollback - Show or trigger rollback

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "agntctl", about = "AgntOS control tool")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Inspect system hardware and state
    Inspect {
        #[arg(default_value = "system")]
        target: String,
    },
    /// Propose a Nix-backed config change
    Propose {
        /// What to change
        description: String,
    },
    /// Apply an approved proposal
    Apply {
        /// Proposal ID to apply
        id: String,
    },
    /// View the activity audit log
    Audit {
        #[arg(default_value = "list")]
        action: String,
    },
    /// Show rollback guidance
    Rollback {
        #[arg(default_value = "list")]
        action: String,
    },
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Command::Inspect { target } => {
            println!("agntctl: inspecting {}", target);
            inspect(target);
        }
        Command::Propose { description } => {
            println!("agntctl: proposing change: {}", description);
            propose(description);
        }
        Command::Apply { id } => {
            println!("agntctl: applying proposal {}", id);
            apply(id);
        }
        Command::Audit { action } => {
            println!("agntctl: audit {}", action);
            audit(action);
        }
        Command::Rollback { action } => {
            println!("agntctl: rollback {}", action);
            rollback(action);
        }
    }
}

fn inspect(target: String) {
    println!("  OS: AgntOS (NixOS-based)");
    println!("  Host: {}", hostname());
    println!("  Kernel: {}", kernel_version());
    println!("  CPU: {}", cpu_info());
    println!("  Memory: {}", memory_info());
    println!("  Target requested: {}", target);
}

fn hostname() -> String {
    std::process::Command::new("hostname")
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .unwrap_or_else(|_| "unknown".into())
}

fn kernel_version() -> String {
    std::process::Command::new("uname")
        .arg("-r")
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .unwrap_or_else(|_| "unknown".into())
}

fn cpu_info() -> String {
    std::fs::read_to_string("/proc/cpuinfo")
        .ok()
        .and_then(|s| {
            s.lines()
                .find(|l| l.starts_with("model name"))
                .map(|l| l.split(':').nth(1).unwrap_or("unknown").trim().to_string())
        })
        .unwrap_or_else(|| "unknown".into())
}

fn memory_info() -> String {
    std::fs::read_to_string("/proc/meminfo")
        .ok()
        .and_then(|s| {
            s.lines()
                .find(|l| l.starts_with("MemTotal"))
                .map(|l| l.split(':').nth(1).unwrap_or("unknown").trim().to_string())
        })
        .unwrap_or_else(|| "unknown".into())
}

fn propose(description: String) {
    println!("  This is a placeholder.");
    println!("  In Phase 1, this will generate a Nix config diff based on: {}", description);
    println!("  Files would be written to: /etc/agntos/");
}

fn apply(id: String) {
    println!("  This is a placeholder.");
    println!("  In Phase 1, this will apply proposal {} to /etc/agntos/ and trigger nixos-rebuild.", id);
}

fn audit(action: String) {
    println!("  This is a placeholder.");
    println!("  In Phase 1, this will read the local audit log in: /var/log/agntos/");
    println!("  Action: {}", action);
}

fn rollback(action: String) {
    println!("  This is a placeholder.");
    println!("  In Phase 1, this will show nixos-rebuild rollback guidance.");
    println!("  Action: {}", action);
}
