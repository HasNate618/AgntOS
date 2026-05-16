// agntctl - AgntOS control tool for Nix-backed system changes.
//
// Commands:
//   inspect  - Read-only system inspection
//   propose  - Plan a config change without applying
//   apply    - Apply an approved proposal
//   audit    - View activity log
//   rollback - Show or trigger rollback

mod apply;
mod audit;
mod inspect;
mod memory;
mod model;
mod propose;
mod rollback;
mod sys;

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

        /// Original user prompt — records the "why" in the audit log
        #[arg(long)]
        prompt: Option<String>,

        /// Agent's rationale for the change
        #[arg(long)]
        rationale: Option<String>,
    },
    /// Apply an approved proposal
    Apply {
        /// Proposal ID (from agntctl propose output)
        #[arg()]
        id: String,

        /// Dry run — preview without writing files
        #[arg(long)]
        dry_run: bool,

        /// Skip nixos-rebuild after writing files
        #[arg(long)]
        no_rebuild: bool,

        /// Persist across reboots (use nixos-rebuild switch instead of test)
        #[arg(long)]
        persist: bool,

        /// Config directory (default: /etc/agntos)
        #[arg(long)]
        config_dir: Option<PathBuf>,
    },
    /// View the activity audit log
    Audit {
        /// Action: list, show <id>, or search <query>
        #[arg(default_value = "list")]
        action: String,

        /// Audit entry ID (for "show")
        #[arg(required = false)]
        id: Option<String>,

        /// Search query (for "search")
        #[arg(long)]
        query: Option<String>,

        /// Max entries to show (for "list" / "search")
        #[arg(long, default_value = "20")]
        limit: usize,

        /// Output as JSON
        #[arg(long)]
        json: bool,

        /// Config directory (default: /var/log/agntos)
        #[arg(long)]
        config_dir: Option<PathBuf>,
    },
    /// View or resolve model routing configuration
    Model {
        /// Action: list, route, add, remove, set-route, or suggest
        #[arg(default_value = "list")]
        action: String,

        /// Profile name
        #[arg(required = false)]
        profile: Option<String>,

        /// Endpoint URL (for add)
        #[arg(long)]
        endpoint: Option<String>,

        /// Model name (for add)
        #[arg(long)]
        model: Option<String>,

        /// API key env var name (for add)
        #[arg(long)]
        api_key_env: Option<String>,

        /// Max tokens (for add)
        #[arg(long)]
        max_tokens: Option<u32>,

        /// Temperature (for add)
        #[arg(long)]
        temperature: Option<f32>,

        /// Task class (for route, set-route)
        #[arg(long)]
        task: Option<String>,

        /// Target profile (for set-route)
        #[arg(long)]
        route_profile: Option<String>,

        /// Output as JSON
        #[arg(long)]
        json: bool,

        /// Config directory (default: /etc/agntos)
        #[arg(long)]
        config_dir: Option<PathBuf>,
    },
    /// Manage Hermes-style memory files
    Memory {
        /// Action: show | add | replace | remove
        #[arg(default_value = "show")]
        action: String,

        /// File: memory or user
        #[arg(required = false)]
        file: Option<String>,

        /// Section name for add
        #[arg(long)]
        section: Option<String>,

        /// Content for add
        #[arg(long)]
        content: Option<String>,

        /// Substring to replace/remove
        #[arg(long)]
        target: Option<String>,

        /// Replacement text for replace
        #[arg(long)]
        replacement: Option<String>,

        /// Output as JSON
        #[arg(long)]
        json: bool,

        /// Config directory (default: /etc/agntos)
        #[arg(long)]
        config_dir: Option<PathBuf>,
    },
    /// Roll back to the previous NixOS generation, or list generations
    Rollback {
        /// Action: list, apply, or undo
        #[arg(default_value = "apply")]
        action: String,

        /// Audit entry ID to undo (for "undo" action)
        #[arg(long)]
        undo_id: Option<String>,

        /// Config directory (default: /etc/agntos)
        #[arg(long)]
        config_dir: Option<PathBuf>,
    },
    /// Read a file's contents
    Read {
        /// File path
        #[arg()]
        path: String,
    },
    /// Create or overwrite a file
    Write {
        /// File path
        #[arg()]
        path: String,

        /// Content to write
        #[arg(long)]
        content: String,

        /// Config directory (default: /etc/agntos)
        #[arg(long)]
        config_dir: Option<PathBuf>,
    },
    /// Replace text in a file
    Edit {
        /// File path
        #[arg()]
        path: String,

        /// String to replace
        #[arg(long)]
        old: String,

        /// Replacement string
        #[arg(long)]
        new: String,

        /// Config directory (default: /etc/agntos)
        #[arg(long)]
        config_dir: Option<PathBuf>,
    },
    /// Execute a shell command via bash -c
    Bash {
        /// Shell command to execute
        #[arg()]
        command: String,

        /// Config directory (default: /etc/agntos)
        #[arg(long)]
        config_dir: Option<PathBuf>,
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
                        println!(
                            "  Cores:  {} physical / {} logical",
                            info.cpu.cores, info.cpu.threads
                        );
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
                        println!(
                            "  Available: {} ({:.0}%)",
                            bytes_human(info.memory.available_kb * 1024),
                            info.memory.available_pct()
                        );
                        println!(
                            "  Swap:      {} total, {} free",
                            bytes_human(info.memory.swap_total_kb * 1024),
                            bytes_human(info.memory.swap_free_kb * 1024)
                        );
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
                            let model = if gpu.model.is_empty() {
                                &gpu.device_id
                            } else {
                                &gpu.model
                            };
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
                            println!(
                                "  {:<8} {:>9} {}",
                                disk.name,
                                bytes_human(disk.size_bytes),
                                disk.mount.as_deref().unwrap_or("")
                            );
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
                            println!(
                                "  {:<8} {}  {}  {}",
                                iface.name,
                                iface.state,
                                iface.mac,
                                iface.ip.as_deref().unwrap_or("")
                            );
                        }
                    }
                }
                other => {
                    eprintln!("Unknown target: {}. Valid targets: system, cpu, memory, gpu, disk, network", other);
                    std::process::exit(1);
                }
            }
        }
        Command::Propose {
            description,
            json,
            dry_run,
            config_dir,
            prompt,
            rationale,
        } => match propose::execute(
            &description,
            dry_run,
            config_dir.as_ref(),
            prompt.as_deref(),
            rationale.as_deref(),
        ) {
            Ok(output) => {
                if json {
                    println!(
                        "{{\"status\": \"ok\", \"output\": {}}}",
                        serde_json::to_string(&output).unwrap()
                    );
                } else {
                    print!("{}", output);
                }
            }
            Err(e) => {
                eprintln!("Error: {}", e);
                std::process::exit(1);
            }
        },
        Command::Apply {
            id,
            dry_run,
            no_rebuild,
            persist,
            config_dir,
        } => match apply::execute(&id, dry_run, no_rebuild, persist, config_dir.as_ref()) {
            Ok(output) => {
                print!("{}", output);
            }
            Err(e) => {
                eprintln!("Error: {}", e);
                std::process::exit(1);
            }
        },
        Command::Audit {
            action,
            id,
            limit,
            json,
            config_dir,
            query,
        } => match action.as_str() {
            "list" => match audit::execute_list(limit, json, config_dir.as_ref()) {
                Ok(output) => print!("{}", output),
                Err(e) => eprintln!("Error: {}", e),
            },
            "show" => {
                let entry_id = id.as_deref().unwrap_or("show");
                match audit::execute_show(entry_id, json, config_dir.as_ref()) {
                    Ok(output) => print!("{}", output),
                    Err(e) => eprintln!("Error: {}", e),
                }
            }
            "search" => {
                let q = query.as_deref().unwrap_or("");
                if q.is_empty() {
                    eprintln!("Usage: audit search <query>");
                    std::process::exit(1);
                }
                match audit::execute_search(q, limit, json, config_dir.as_ref()) {
                    Ok(output) => print!("{}", output),
                    Err(e) => eprintln!("Error: {}", e),
                }
            }
            other => {
                eprintln!(
                    "Unknown audit action: {}. Use 'list', 'show <id>', or 'search <query>'",
                    other
                );
                std::process::exit(1);
            }
        },
        Command::Model {
            action,
            profile,
            endpoint,
            model,
            api_key_env,
            max_tokens,
            temperature,
            task,
            route_profile,
            json,
            config_dir,
        } => {
            let result = match action.as_str() {
                "list" => model::execute_list(json, config_dir.as_ref()),
                "route" => {
                    match task.as_deref() {
                        Some(t) => model::execute_route(t, json, config_dir.as_ref()),
                        None => Err("Usage: agntctl model route <task>".to_string()),
                    }
                }
                "add" => {
                    let name = profile.as_deref();
                    let ep = endpoint.as_deref();
                    let mdl = model.as_deref();
                    match (name, ep, mdl) {
                        (Some(n), Some(e), Some(m)) => model::execute_add(n, e, m, api_key_env.as_deref(), max_tokens, temperature, config_dir.as_ref()),
                        _ => Err("Usage: agntctl model add <name> --endpoint <url> --model <name>".to_string()),
                    }
                }
                "remove" => {
                    match profile.as_deref() {
                        Some(n) => model::execute_remove(n, config_dir.as_ref()),
                        None => Err("Usage: agntctl model remove <name>".to_string()),
                    }
                }
                "set-route" => {
                    let t = task.as_deref();
                    let p = route_profile.as_deref();
                    match (t, p) {
                        (Some(t), Some(p)) => model::execute_set_route(t, p, config_dir.as_ref()),
                        _ => Err("Usage: agntctl model set-route --task <task> --route-profile <profile>".to_string()),
                    }
                }
                "suggest" => model::execute_suggest(config_dir.as_ref()),
                other => Err(format!(
                    "Unknown model action: {}. Use 'list', 'route <task>', 'add', 'remove', 'set-route', or 'suggest'",
                    other
                )),
            };

            match result {
                Ok(output) => print!("{}", output),
                Err(e) => {
                    eprintln!("Error: {}", e);
                    std::process::exit(1);
                }
            }
        }
        Command::Memory {
            action,
            file,
            section,
            content,
            target,
            replacement,
            json,
            config_dir,
        } => {
            let result = match action.as_str() {
                "show" => memory::execute_show(file.as_deref(), json, config_dir.as_ref()),
                "add" => {
                    let file = file
                        .as_deref()
                        .ok_or_else(|| "Usage: agntctl memory add <memory|user> --section <name> --content <text>".to_string());
                    let section = section
                        .as_deref()
                        .ok_or_else(|| "Missing --section for memory add".to_string());
                    let content = content
                        .as_deref()
                        .ok_or_else(|| "Missing --content for memory add".to_string());

                    match (file, section, content) {
                        (Ok(f), Ok(s), Ok(c)) => memory::execute_add(f, s, c, config_dir.as_ref()),
                        (Err(e), _, _) | (_, Err(e), _) | (_, _, Err(e)) => Err(e),
                    }
                }
                "replace" => {
                    let file = file
                        .as_deref()
                        .ok_or_else(|| "Usage: agntctl memory replace <memory|user> --target <substring> --replacement <text>".to_string());
                    let target = target
                        .as_deref()
                        .ok_or_else(|| "Missing --target for memory replace".to_string());
                    let replacement = replacement
                        .as_deref()
                        .ok_or_else(|| "Missing --replacement for memory replace".to_string());

                    match (file, target, replacement) {
                        (Ok(f), Ok(t), Ok(r)) => {
                            memory::execute_replace(f, t, r, config_dir.as_ref())
                        }
                        (Err(e), _, _) | (_, Err(e), _) | (_, _, Err(e)) => Err(e),
                    }
                }
                "remove" => {
                    let file = file.as_deref().ok_or_else(|| {
                        "Usage: agntctl memory remove <memory|user> --target <substring>"
                            .to_string()
                    });
                    let target = target
                        .as_deref()
                        .ok_or_else(|| "Missing --target for memory remove".to_string());

                    match (file, target) {
                        (Ok(f), Ok(t)) => memory::execute_remove(f, t, config_dir.as_ref()),
                        (Err(e), _) | (_, Err(e)) => Err(e),
                    }
                }
                "consolidate" => {
                    let file = file.as_deref().unwrap_or("memory");
                    memory::execute_consolidate(file, config_dir.as_ref())
                }
                other => Err(format!(
                    "Unknown memory action: {}. Use show/add/replace/remove/consolidate",
                    other
                )),
            };

            match result {
                Ok(output) => print!("{}", output),
                Err(e) => {
                    eprintln!("Error: {}", e);
                    std::process::exit(1);
                }
            }
        }
        Command::Rollback {
            action,
            undo_id,
            config_dir,
        } => {
            let result = match action.as_str() {
                "list" | "list-generations" => rollback::execute_list(config_dir.as_ref()),
                "apply" | "do" | "" => rollback::execute(config_dir.as_ref()),
                "undo" | "surgical" => {
                    rollback::execute_undo(config_dir.as_ref(), undo_id.as_deref())
                }
                other => Err(format!(
                    "Unknown rollback action: {}. Use 'list', 'apply', or 'undo'",
                    other
                )),
            };
            match result {
                Ok(output) => print!("{}", output),
                Err(e) => {
                    eprintln!("Error: {}", e);
                    std::process::exit(1);
                }
            }
        }
        Command::Read { path } => match sys::execute_read(&path) {
            Ok(content) => print!("{}", content),
            Err(e) => {
                eprintln!("Error: {}", e);
                std::process::exit(1);
            }
        },
        Command::Write {
            path,
            content,
            config_dir,
        } => match sys::execute_write(&path, &content, config_dir.as_ref()) {
            Ok(out) => print!("{}", out),
            Err(e) => {
                eprintln!("Error: {}", e);
                std::process::exit(1);
            }
        },
        Command::Edit {
            path,
            old,
            new,
            config_dir,
        } => match sys::execute_edit(&path, &old, &new, config_dir.as_ref()) {
            Ok(out) => print!("{}", out),
            Err(e) => {
                eprintln!("Error: {}", e);
                std::process::exit(1);
            }
        },
        Command::Bash {
            command,
            config_dir,
        } => match sys::execute_bash(&command, config_dir.as_ref()) {
            Ok(out) => print!("{}", out),
            Err(e) => {
                eprintln!("Error: {}", e);
                std::process::exit(1);
            }
        },
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
