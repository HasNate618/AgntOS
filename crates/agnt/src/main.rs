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
    #[command(about = "Interactive chat (foreground agntd)")]
    Chat,
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
        None | Some(Commands::Chat) => exec("agntd", &[]),
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
