use crate::llm::LlmClient;
use crate::util;
use agnt_common::wire::ServerMessage;
use serde::Deserialize;
use serde_json::json;
use std::io::Write;
use std::sync::atomic::{AtomicU64, AtomicU8, Ordering};
use tokio::sync::broadcast;

pub type EventSender = broadcast::Sender<ServerMessage>;

pub fn create_event_channel() -> EventSender {
    let (tx, _) = broadcast::channel(32);
    tx
}

fn default_interval() -> u64 {
    300
}
fn default_disk() -> u8 {
    95
}

#[derive(Debug, Deserialize)]
struct WatchdogConfig {
    #[serde(default = "default_interval")]
    interval_secs: u64,
    #[serde(default = "default_disk")]
    disk_threshold_pct: u8,
}

impl Default for WatchdogConfig {
    fn default() -> Self {
        Self {
            interval_secs: 300,
            disk_threshold_pct: 95,
        }
    }
}

static POLL_INTERVAL: AtomicU64 = AtomicU64::new(300);
static DISK_THRESHOLD: AtomicU8 = AtomicU8::new(95);

struct Check {
    name: &'static str,
    command: &'static str,
    tripped_if: fn(&str) -> bool,
    fetch_logs: fn(&str) -> String,
}

static CHECKS: &[Check] = &[
    Check {
        name: "failed_systemd_units",
        command: "systemctl --failed --plain --no-legend 2>/dev/null || true",
        tripped_if: |output| !output.trim().is_empty(),
        fetch_logs: |stdout| {
            let units: Vec<&str> = stdout
                .lines()
                .filter_map(|l| l.split_whitespace().next())
                .collect();
            let mut logs = String::new();
            for unit in units.iter().take(5) {
                let cmd = format!(
                    "journalctl -u {} -n 50 --no-pager 2>/dev/null || true",
                    unit
                );
                if let Ok((out, _, _)) =
                    util::run_agntctl(&["bash", &cmd, "--config-dir", "/etc/agntos"])
                {
                    logs.push_str(&format!("=== {} ===\n{}\n", unit, out));
                }
            }
            logs
        },
    },
    Check {
        name: "disk_critical",
        command: "df -h / | tail -1 | awk '{print $5}' | sed 's/%//'",
        tripped_if: |output| {
            let threshold = DISK_THRESHOLD.load(Ordering::Relaxed);
            output
                .trim()
                .parse::<u8>()
                .ok()
                .map_or(false, |pct| pct >= threshold)
        },
        fetch_logs: |_| String::new(),
    },
    Check {
        name: "oom_killer",
        command: "dmesg 2>/dev/null | grep -i oom || true",
        tripped_if: |output| !output.trim().is_empty(),
        fetch_logs: |stdout| {
            let lines: Vec<&str> = stdout.lines().rev().take(10).collect();
            lines.join("\n")
        },
    },
];

pub fn start(config_dir: String) {
    std::thread::spawn(move || {
        let wd_cfg = load_config(&config_dir);
        POLL_INTERVAL.store(wd_cfg.interval_secs, Ordering::Relaxed);
        DISK_THRESHOLD.store(wd_cfg.disk_threshold_pct, Ordering::Relaxed);

        eprintln!(
            "[watchdog] config: interval={}s, disk_threshold={}%",
            wd_cfg.interval_secs, wd_cfg.disk_threshold_pct
        );

        let rt = match tokio::runtime::Runtime::new() {
            Ok(r) => r,
            Err(e) => {
                eprintln!("[watchdog] failed to create runtime: {}", e);
                return;
            }
        };

        let client = match LlmClient::from_config(&config_dir, "watchdog") {
            Ok(c) => c,
            Err(e) => {
                eprintln!("[watchdog] LLM not available: {}. Watchdog disabled.", e);
                return;
            }
        };

        let interval = POLL_INTERVAL.load(Ordering::Relaxed);
        rt.block_on(async move {
            let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(interval));
            interval.tick().await;
            loop {
                interval.tick().await;
                if let Err(e) = run_cycle(&client, &config_dir).await {
                    eprintln!("[watchdog] cycle error: {}", e);
                }
            }
        });
    });
}

fn load_config(config_dir: &str) -> WatchdogConfig {
    let path = format!("{}/watchdog.toml", config_dir);
    match std::fs::read_to_string(&path) {
        Ok(content) => toml::from_str(&content).unwrap_or_else(|e| {
            eprintln!(
                "[watchdog] invalid watchdog.toml ({}), using defaults: {}",
                path, e
            );
            WatchdogConfig::default()
        }),
        Err(_) => WatchdogConfig::default(),
    }
}

async fn run_cycle(client: &LlmClient, config_dir: &str) -> Result<(), String> {
    for check in CHECKS {
        let result = util::run_agntctl(&["bash", check.command, "--config-dir", config_dir]);

        let stdout = match result {
            Ok((out, _, _)) => out,
            Err(e) => {
                eprintln!("[watchdog] check '{}' command failed: {}", check.name, e);
                continue;
            }
        };

        if !(check.tripped_if)(&stdout) {
            continue;
        }

        log(
            config_dir,
            &format!("[watchdog] {}: check tripped", check.name),
        );

        let logs = (check.fetch_logs)(&stdout);
        triage(client, config_dir, check, &stdout, &logs).await;
    }
    Ok(())
}

async fn triage(
    client: &LlmClient,
    config_dir: &str,
    check: &Check,
    check_output: &str,
    logs: &str,
) {
    let prompt = format!(
        "You are an NixOS diagnostics assistant. Given this health check result and logs, \
classify the issue into exactly one category. Respond with a SINGLE LINE:\n\n\
CONFIG_ERROR: <short description of the NixOS config change needed>\n\
TRANSIENT: <brief reason>\n\
UNKNOWN: <note>\n\n\
Health check: {}\nOutput:\n{}\n\nLogs:\n{}",
        check.name, check_output, logs
    );

    let messages = vec![
        json!({"role": "system", "content": "You classify system issues as CONFIG_ERROR, TRANSIENT, or UNKNOWN. Respond with exactly one line."}),
        json!({"role": "user", "content": prompt}),
    ];

    let tools: Vec<serde_json::Value> = vec![];
    let response = match client.complete(&messages, &tools).await {
        Ok(r) => r.content,
        Err(e) => {
            eprintln!("[watchdog] LLM triage failed: {}", e);
            return;
        }
    };

    let line = response.lines().next().unwrap_or("").trim();
    if line.is_empty() {
        return;
    }

    if let Some(desc) = line.strip_prefix("CONFIG_ERROR:") {
        let description = desc.trim();
        log(
            config_dir,
            &format!("[watchdog] Config error detected: {}", description),
        );
        let propose_desc = format!("automated fix: {}", description);
        match util::run_agntctl(&["propose", "--config-dir", config_dir, &propose_desc]) {
            Ok((stdout, _, _)) => {
                log(config_dir, &format!("[watchdog] Fix drafted:\n{}", stdout));
            }
            Err(e) => {
                eprintln!("[watchdog] failed to draft proposal: {}", e);
            }
        }
    } else if line.starts_with("TRANSIENT") {
        eprintln!("[watchdog] {} transient: {}", check.name, line);
    } else {
        eprintln!("[watchdog] {} unclassified: {}", check.name, line);
    }
}

fn log(config_dir: &str, msg: &str) {
    eprintln!("{}", msg);
    let log_path = format!("{}/memory/watchdog.log", config_dir);
    if let Ok(ts) = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH) {
        let line = format!("{} {}\n", ts.as_secs(), msg);
        let _ = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&log_path)
            .and_then(|mut f| f.write_all(line.as_bytes()));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn disk_threshold_atomic_scenarios() {
        DISK_THRESHOLD.store(95, Ordering::Relaxed);
        let check = &CHECKS[1];
        assert!((check.tripped_if)("95"));
        assert!((check.tripped_if)("99"));
        assert!(!(check.tripped_if)("50"));
        assert!(!(check.tripped_if)("94"));
        assert!(!(check.tripped_if)(""));
        assert!(!(check.tripped_if)("abc"));

        DISK_THRESHOLD.store(90, Ordering::Relaxed);
        assert!((check.tripped_if)("95"));
        assert!((check.tripped_if)("90"));
        assert!(!(check.tripped_if)("89"));

        DISK_THRESHOLD.store(50, Ordering::Relaxed);
        assert!((check.tripped_if)("55"));
        assert!(!(check.tripped_if)("45"));

        DISK_THRESHOLD.store(80, Ordering::Relaxed);
        assert!((check.tripped_if)("90"));
        assert!(!(check.tripped_if)("70"));

        DISK_THRESHOLD.store(95, Ordering::Relaxed);
    }

    #[test]
    fn failed_units_trips_on_nonempty() {
        let check = &CHECKS[0];
        assert!((check.tripped_if)("nginx.service loaded failed"));
        assert!(!(check.tripped_if)(""));
        assert!(!(check.tripped_if)("   \n   "));
    }

    #[test]
    fn oom_trips_on_nonempty() {
        let check = &CHECKS[2];
        assert!((check.tripped_if)("Out of memory: Killed process"));
        assert!(!(check.tripped_if)(""));
    }

    #[test]
    fn config_loads_defaults_when_no_file() {
        let cfg = load_config("/tmp/nonexistent-agntos-dir");
        assert_eq!(cfg.interval_secs, 300);
        assert_eq!(cfg.disk_threshold_pct, 95);
    }

    #[test]
    fn config_loads_from_valid_toml() {
        let dir = std::env::temp_dir().join("agntos-watchdog-config-test");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(
            dir.join("watchdog.toml"),
            "interval_secs = 60\ndisk_threshold_pct = 85\n",
        )
        .unwrap();
        let cfg = load_config(&dir.to_string_lossy());
        assert_eq!(cfg.interval_secs, 60);
        assert_eq!(cfg.disk_threshold_pct, 85);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn config_falls_back_on_invalid_toml() {
        let dir = std::env::temp_dir().join("agntos-watchdog-config-bad-test");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("watchdog.toml"), "interval_secs = not-a-number\n").unwrap();
        let cfg = load_config(&dir.to_string_lossy());
        assert_eq!(cfg.interval_secs, 300);
        assert_eq!(cfg.disk_threshold_pct, 95);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn config_accepts_partial_overrides() {
        let dir = std::env::temp_dir().join("agntos-watchdog-partial-test");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("watchdog.toml"), "interval_secs = 120\n").unwrap();
        let cfg = load_config(&dir.to_string_lossy());
        assert_eq!(cfg.interval_secs, 120);
        assert_eq!(cfg.disk_threshold_pct, 95);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn config_ignores_unknown_keys() {
        let dir = std::env::temp_dir().join("agntos-watchdog-unknown-test");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(
            dir.join("watchdog.toml"),
            "interval_secs = 60\ndisk_threshold_pct = 90\nnotakey = true\n",
        )
        .unwrap();
        let cfg = load_config(&dir.to_string_lossy());
        assert_eq!(cfg.interval_secs, 60);
        assert_eq!(cfg.disk_threshold_pct, 90);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn poll_interval_atomic_stores_and_loads() {
        POLL_INTERVAL.store(60, Ordering::Relaxed);
        assert_eq!(POLL_INTERVAL.load(Ordering::Relaxed), 60);
        POLL_INTERVAL.store(300, Ordering::Relaxed);
        assert_eq!(POLL_INTERVAL.load(Ordering::Relaxed), 300);
        POLL_INTERVAL.store(0, Ordering::Relaxed);
        assert_eq!(POLL_INTERVAL.load(Ordering::Relaxed), 0);
        POLL_INTERVAL.store(300, Ordering::Relaxed);
    }
}
