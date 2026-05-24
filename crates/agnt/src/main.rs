mod markdown;
mod skills;
mod socket;
mod socket_chat;
mod tui;

use clap::{Parser, Subcommand};
use std::process::{Command, Stdio};

#[derive(Parser)]
#[command(name = "agnt", about = "AgntOS — local agent CLI")]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    #[command(about = "Interactive chat (TUI when terminal, else plain REPL)")]
    Chat {
        #[arg(long)]
        socket: Option<String>,
        #[arg(long, help = "Line-oriented REPL instead of ratatui TUI")]
        plain: bool,
        #[arg(long, help = "Always run foreground agntd instead of connecting")]
        foreground: bool,
    },
    #[command(about = "Run agntd in the foreground")]
    Daemon {
        #[arg(long, default_value = "/run/agntd/agent.sock")]
        socket: String,
    },
    #[command(about = "OS control commands (agntctl)")]
    System {
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },
}

fn main() {
    let cli = Cli::parse();
    let status = match cli.command {
        None => run_chat(None, false, false),
        Some(Commands::Chat {
            socket,
            plain,
            foreground,
        }) => run_chat(socket, plain, foreground),
        Some(Commands::Daemon { socket }) => exec("agntd", &["--socket", &socket]),
        Some(Commands::System { args }) => {
            let mut argv: Vec<&str> = args.iter().map(String::as_str).collect();
            if argv.is_empty() {
                argv.push("--help");
            }
            exec("agntctl", &argv)
        }
    };
    std::process::exit(status);
}

fn run_chat(socket: Option<String>, plain: bool, foreground: bool) -> i32 {
    if foreground {
        return exec("agntd", &[]);
    }
    let path = socket.unwrap_or_else(socket::default_socket_path);
    if !socket::socket_available(&path) {
        eprintln!("agnt: no daemon at {} — starting foreground agntd", path);
        return exec("agntd", &[]);
    }
    let result = if tui::should_use_tui(plain) {
        tui::run(&path)
    } else {
        socket_chat::run(&path)
    };
    match result {
        Ok(()) => 0,
        Err(e) if e == "quit" => 0,
        Err(e) => {
            eprintln!("agnt: {}", e);
            1
        }
    }
}

fn exec(bin: &str, args: &[&str]) -> i32 {
    let status = Command::new(bin)
        .args(args)
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status();
    match status {
        Ok(s) => s.code().unwrap_or(1),
        Err(e) => {
            eprintln!("agnt: failed to run {}: {}", bin, e);
            127
        }
    }
}
