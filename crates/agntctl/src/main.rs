// agntctl - AgntOS control tool for Nix-backed system changes.
//
// Commands:
//   inspect  - Read-only system inspection
//   propose  - Plan a config change without applying
//   apply    - Apply an approved proposal
//   audit    - View activity log
//   rollback - Show or trigger rollback

mod inspect;
mod propose;

use clap::{Parser, Subcommand};
use std::path::PathBuf;

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
        /// What to inspect (system, cpu, memory, gpu, disk, network)
        #[arg(default_value = "system")]
        target: String,

        /// Output as JSON
        #[arg(long)]
        json: bool,
    },
    /// Propose a Nix-backed config change
    Propose {
        /// Description of the change (e.g. "install firefox", "enable docker")
        #[arg()]
        description: String,

        /// Output as JSON
        #[arg(long)]
        json: bool,

        /// Dry run — generate but don't save
        #[arg(long)]
        dry_run: bool,

        /// Config directory (default: /etc/agntos)
        #[arg(long)]
        config_dir: Option<PathBuf>,
    },
    /// Apply an approved proposal
    Apply {
        #[arg()]
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
        Command::Inspect { target, json } => {
            match target.as_str() {
                "system" | "all" => {
                    let info = inspect::SystemInfo::collect();
                    if json {
                        println!("{}", serde_json::to_string_pretty(&info).unwrap());
                    } else {
                        print!("{}", info.display());
                    }
                }
                "cpu" => {
                    let info = inspect::SystemInfo::collect();
                    if json {
                        println!("{}", serde_json::to_string_pretty(&info.cpu).unwrap());
                    } else {
                        println!("CPU:");
                        println!("  Model:  {}", info.cpu.model);
                        println!("  Cores:  {} physical / {} logical", info.cpu.cores, info.cpu.threads);
                        println!("  Arch:   {}", info.cpu.arch);
                        if let Some(ref freq) = info.cpu.max_freq {
                            println!("  Max:    {} MHz", freq);
                        }
                    }
                }
                "memory" | "mem" => {
                    let info = inspect::SystemInfo::collect();
                    if json {
                        println!("{}", serde_json::to_string_pretty(&info.memory).unwrap());
                    } else {
                        println!("Memory:");
                        println!("  Total:     {}", bytes_human(info.memory.total_kb * 1024));
                        println!("  Available: {} ({:.0}%)",
                            bytes_human(info.memory.available_kb * 1024),
                            info.memory.available_pct());
                        println!("  Swap:      {} total, {} free",
                            bytes_human(info.memory.swap_total_kb * 1024),
                            bytes_human(info.memory.swap_free_kb * 1024));
                    }
                }
                "gpu" => {
                    let info = inspect::SystemInfo::collect();
                    if json {
                        println!("{}", serde_json::to_string_pretty(&info.gpu).unwrap());
                    } else if info.gpu.is_empty() {
                        println!("No GPU detected.");
                    } else {
                        println!("GPU:");
                        for gpu in &info.gpu {
                            let model = if gpu.model.is_empty() { &gpu.device_id } else { &gpu.model };
                            println!("  {}: {} (driver: {})", gpu.vendor, model, gpu.driver);
                        }
                    }
                }
                "disk" | "disks" => {
                    let info = inspect::SystemInfo::collect();
                    if json {
                        println!("{}", serde_json::to_string_pretty(&info.disks).unwrap());
                    } else if info.disks.is_empty() {
                        println!("No disks detected.");
                    } else {
                        println!("Disks:");
                        for disk in &info.disks {
                            println!("  {:<8} {:>9} {}", disk.name, bytes_human(disk.size_bytes), disk.mount.as_deref().unwrap_or(""));
                        }
                    }
                }
                "network" | "net" => {
                    let info = inspect::SystemInfo::collect();
                    if json {
                        println!("{}", serde_json::to_string_pretty(&info.network).unwrap());
                    } else if info.network.is_empty() {
                        println!("No network interfaces detected.");
                    } else {
                        println!("Network:");
                        for iface in &info.network {
                            println!("  {:<8} {}  {}  {}", iface.name, iface.state, iface.mac, iface.ip.as_deref().unwrap_or(""));
                        }
                    }
                }
                other => {
                    eprintln!("Unknown target: {}. Valid targets: system, cpu, memory, gpu, disk, network", other);
                    std::process::exit(1);
                }
            }
        }
        Command::Propose { description, json, dry_run, config_dir } => {
            match propose::execute(&description, dry_run, config_dir.as_ref()) {
                Ok(output) => {
                    if json {
                        println!("{{\"status\": \"ok\", \"output\": {}}}", serde_json::to_string(&output).unwrap());
                    } else {
                        print!("{}", output);
                    }
                }
                Err(e) => {
                    eprintln!("Error: {}", e);
                    std::process::exit(1);
                }
            }
        }
        Command::Apply { id } => {
            println!("agntctl apply {}", id);
            println!("  Not yet implemented.");
        }
        Command::Audit { action } => {
            println!("agntctl audit {}", action);
            println!("  Not yet implemented.");
        }
        Command::Rollback { action } => {
            println!("agntctl rollback {}", action);
            println!("  Not yet implemented.");
        }
    }
}

fn bytes_human(bytes: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
    let mut size = bytes as f64;
    let mut unit = 0;
    while size >= 1024.0 && unit < UNITS.len() - 1 {
        size /= 1024.0;
        unit += 1;
    }
    format!("{:.1} {}", size, UNITS[unit])
}
