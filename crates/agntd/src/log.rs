use agnt_common::paths::agent_state_dir;
use chrono::Local;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::PathBuf;
use std::sync::Mutex;

static LOG_PATH: Mutex<Option<PathBuf>> = Mutex::new(None);

pub fn log_path() -> PathBuf {
    agent_state_dir().join("agntd.log")
}

fn ensure_path() -> PathBuf {
    let mut guard = LOG_PATH.lock().unwrap();
    if let Some(p) = guard.as_ref() {
        return p.clone();
    }
    let path = log_path();
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    *guard = Some(path.clone());
    path
}

pub fn write(level: &str, msg: &str) {
    let line = format!("{} [{}] {}\n", Local::now().format("%Y-%m-%d %H:%M:%S"), level, msg);
    eprint!("{}", line);
    let path = ensure_path();
    if let Ok(mut f) = OpenOptions::new().create(true).append(true).open(&path) {
        let _ = f.write_all(line.as_bytes());
    }
}

pub fn info(msg: &str) {
    write("INFO", msg);
}

pub fn warn(msg: &str) {
    write("WARN", msg);
}

pub fn error(msg: &str) {
    write("ERROR", msg);
}
