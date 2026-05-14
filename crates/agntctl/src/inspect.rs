use serde::Serialize;
use std::path::Path;
use std::time::Duration;

// ── Top-level system info ──

#[derive(Debug, Clone, Serialize)]
pub struct SystemInfo {
    pub hostname: String,
    pub os: OsInfo,
    pub kernel: KernelInfo,
    pub cpu: CpuInfo,
    pub memory: MemoryInfo,
    pub gpu: Vec<GpuInfo>,
    pub disks: Vec<DiskInfo>,
    pub network: Vec<NetworkInfo>,
    pub uptime: Option<UptimeInfo>,
}

impl SystemInfo {
    pub fn collect() -> Self {
        Self {
            hostname: hostname(),
            os: os_info(),
            kernel: kernel_info(),
            cpu: cpu_info(),
            memory: memory_info(),
            gpu: gpu_info(),
            disks: disk_info(),
            network: network_info(),
            uptime: uptime_info(),
        }
    }

    pub fn display(&self) -> String {
        let mut out = String::new();

        out.push_str(&format!(
            "{} v{} {}\n",
            self.os.name, self.os.version_id, self.os.id
        ));
        out.push_str(&format!("  Hostname: {}\n", self.hostname));
        out.push_str(&format!(
            "  Kernel:   {} ({})\n",
            self.kernel.release, self.kernel.arch
        ));
        out.push_str(&format!(
            "  Uptime:   {}\n",
            self.uptime.as_ref().map(|u| u.human()).unwrap_or_default()
        ));

        out.push_str(&format!("\nCPU:\n"));
        out.push_str(&format!("  Model:  {}\n", self.cpu.model));
        out.push_str(&format!(
            "  Cores:  {} physical / {} logical\n",
            self.cpu.cores, self.cpu.threads
        ));
        out.push_str(&format!("  Arch:   {}\n", self.cpu.arch));
        if let Some(ref freq) = self.cpu.max_freq {
            out.push_str(&format!("  Max:    {} MHz\n", freq));
        }

        out.push_str(&format!("\nMemory:\n"));
        out.push_str(&format!(
            "  Total:     {}\n",
            bytes_human(self.memory.total_kb * 1024)
        ));
        out.push_str(&format!(
            "  Available: {} ({:.0}%)\n",
            bytes_human(self.memory.available_kb * 1024),
            self.memory.available_pct()
        ));
        out.push_str(&format!(
            "  Swap:      {} total, {} free\n",
            bytes_human(self.memory.swap_total_kb * 1024),
            bytes_human(self.memory.swap_free_kb * 1024)
        ));

        if !self.gpu.is_empty() {
            out.push_str(&format!("\nGPU:\n"));
            for gpu in &self.gpu {
                out.push_str(&format!(
                    "  {}: {} (driver: {})\n",
                    gpu.vendor, gpu.model, gpu.driver
                ));
            }
        }

        if !self.disks.is_empty() {
            out.push_str(&format!("\nDisks:\n"));
            for disk in &self.disks {
                out.push_str(&format!(
                    "  {:<8} {:>8} {}\n",
                    disk.name,
                    bytes_human(disk.size_bytes),
                    disk.mount.as_deref().unwrap_or("")
                ));
            }
        }

        if !self.network.is_empty() {
            out.push_str(&format!("\nNetwork:\n"));
            for iface in &self.network {
                out.push_str(&format!(
                    "  {:<8} {}  {}  {}\n",
                    iface.name,
                    iface.state,
                    iface.mac,
                    iface.ip.as_deref().unwrap_or("")
                ));
            }
        }

        out
    }
}

// ── Sub-structs ──

#[derive(Debug, Clone, Serialize)]
pub struct OsInfo {
    pub name: String,
    pub version: String,
    pub version_id: String,
    pub id: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct KernelInfo {
    pub release: String,
    pub version: String,
    pub arch: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct CpuInfo {
    pub model: String,
    pub arch: String,
    pub cores: u32,
    pub threads: u32,
    pub max_freq: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct MemoryInfo {
    pub total_kb: u64,
    pub available_kb: u64,
    pub free_kb: u64,
    pub swap_total_kb: u64,
    pub swap_free_kb: u64,
}

impl MemoryInfo {
    pub fn available_pct(&self) -> f64 {
        if self.total_kb > 0 {
            (self.available_kb as f64 / self.total_kb as f64) * 100.0
        } else {
            0.0
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct GpuInfo {
    pub vendor: String,
    pub vendor_id: String,
    pub model: String,
    pub device_id: String,
    pub driver: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct DiskInfo {
    pub name: String,
    pub size_bytes: u64,
    pub mount: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct NetworkInfo {
    pub name: String,
    pub mac: String,
    pub ip: Option<String>,
    pub state: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct UptimeInfo {
    pub seconds: f64,
}

impl UptimeInfo {
    pub fn human(&self) -> String {
        let d = Duration::from_secs_f64(self.seconds);
        let days = d.as_secs() / 86400;
        let hours = (d.as_secs() % 86400) / 3600;
        let mins = (d.as_secs() % 3600) / 60;
        format!("{}d {}h {}m", days, hours, mins)
    }
}

// ── Readers ──

fn hostname() -> String {
    read_first_line("/proc/sys/kernel/hostname")
        .or_else(|| cmd_output("hostname"))
        .unwrap_or_default()
}

fn os_info() -> OsInfo {
    let data = read_keyval_file("/etc/os-release");
    OsInfo {
        name: data
            .get("PRETTY_NAME")
            .or_else(|| data.get("NAME"))
            .cloned()
            .unwrap_or_default(),
        version: data.get("VERSION").cloned().unwrap_or_default(),
        version_id: data.get("VERSION_ID").cloned().unwrap_or_default(),
        id: data.get("ID").cloned().unwrap_or_default(),
    }
}

fn kernel_info() -> KernelInfo {
    KernelInfo {
        release: read_first_line("/proc/sys/kernel/osrelease")
            .unwrap_or_else(|| cmd_output("uname -r").unwrap_or_default()),
        version: read_first_line("/proc/sys/kernel/version").unwrap_or_default(),
        arch: cmd_output("uname -m").unwrap_or_default(),
    }
}

fn cpu_info() -> CpuInfo {
    let data = read_colon_file("/proc/cpuinfo");
    let model = data.get("model name").cloned().unwrap_or_default();
    let cores = data
        .get("cpu cores")
        .and_then(|s| s.parse().ok())
        .unwrap_or(0);
    let siblings = data
        .get("siblings")
        .and_then(|s| s.parse().ok())
        .unwrap_or(0);
    let freq = data.get("cpu MHz").cloned();
    let arch = cmd_output("uname -m").unwrap_or_default();

    CpuInfo {
        model,
        arch,
        cores,
        threads: siblings,
        max_freq: freq,
    }
}

fn memory_info() -> MemoryInfo {
    let data = read_colon_file("/proc/meminfo");
    MemoryInfo {
        total_kb: data
            .get("MemTotal")
            .map(|s| parse_kb(s))
            .flatten()
            .unwrap_or(0),
        available_kb: data
            .get("MemAvailable")
            .map(|s| parse_kb(s))
            .flatten()
            .unwrap_or(0),
        free_kb: data
            .get("MemFree")
            .map(|s| parse_kb(s))
            .flatten()
            .unwrap_or(0),
        swap_total_kb: data
            .get("SwapTotal")
            .map(|s| parse_kb(s))
            .flatten()
            .unwrap_or(0),
        swap_free_kb: data
            .get("SwapFree")
            .map(|s| parse_kb(s))
            .flatten()
            .unwrap_or(0),
    }
}

fn gpu_info() -> Vec<GpuInfo> {
    let mut gpus = Vec::new();

    // Try reading from /sys/class/drm/
    if let Ok(entries) = std::fs::read_dir("/sys/class/drm") {
        for entry in entries.flatten() {
            let name = entry.file_name();
            let name = name.to_string_lossy();
            if !name.starts_with("card") || name.contains('-') {
                continue; // skip render nodes and connectors
            }
            let dev_path = entry.path().join("device");
            if !dev_path.exists() {
                continue;
            }

            let vendor_id = read_first_line(&dev_path.join("vendor")).unwrap_or_default();
            let device_id = read_first_line(&dev_path.join("device")).unwrap_or_default();
            let driver = std::fs::read_link(&dev_path.join("driver"))
                .ok()
                .and_then(|p| p.file_name().map(|s| s.to_string_lossy().to_string()))
                .unwrap_or_default();

            // Try to get a human-readable name via lspci
            let model = if !vendor_id.is_empty() && !device_id.is_empty() {
                cmd_output(&format!(
                    "lspci -d {}:{} -mm",
                    vendor_id.trim_start_matches("0x"),
                    device_id.trim_start_matches("0x")
                ))
                .unwrap_or_default()
            } else {
                String::new()
            };

            gpus.push(GpuInfo {
                vendor: pci_vendor_name(&vendor_id).to_string(),
                vendor_id: vendor_id.trim().to_string(),
                model: model.trim().to_string(),
                device_id: device_id.trim().to_string(),
                driver,
            });
        }
    }

    // Fallback: try lspci
    if gpus.is_empty() {
        if let Some(output) = cmd_output_opt("lspci -nn") {
            for line in output.lines() {
                let lower = line.to_lowercase();
                if !(lower.contains("vga") || lower.contains("3d") || lower.contains("display")) {
                    continue;
                }
                gpus.push(GpuInfo {
                    vendor: String::new(),
                    vendor_id: String::new(),
                    model: line.trim().to_string(),
                    device_id: String::new(),
                    driver: String::new(),
                });
            }
        }
    }

    gpus
}

fn disk_info() -> Vec<DiskInfo> {
    let mut disks = Vec::new();
    if let Ok(entries) = std::fs::read_dir("/sys/block") {
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().to_string();
            // Skip loop, ram, dm, zram devices
            if name.starts_with("loop")
                || name.starts_with("ram")
                || name.starts_with("dm-")
                || name.starts_with("zram")
            {
                continue;
            }
            let size_str = read_first_line(&entry.path().join("size")).unwrap_or_default();
            let sectors: u64 = size_str.trim().parse().unwrap_or(0);
            let size_bytes = sectors * 512;

            // Find mount point via /proc/mounts
            let mount = find_mount(&name);

            disks.push(DiskInfo {
                name,
                size_bytes,
                mount,
            });
        }
    }
    disks.sort_by(|a, b| b.size_bytes.cmp(&a.size_bytes));
    disks
}

fn network_info() -> Vec<NetworkInfo> {
    let mut ifaces = Vec::new();
    if let Ok(entries) = std::fs::read_dir("/sys/class/net") {
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().to_string();
            if name == "lo" {
                continue;
            }

            let mac = read_first_line(&entry.path().join("address")).unwrap_or_default();
            let state = match read_first_line(&entry.path().join("operstate")).as_deref() {
                Some("up") => "UP",
                _ => "DOWN",
            };

            // Try to find IP
            let ip = find_ip(&name);

            ifaces.push(NetworkInfo {
                name,
                mac: mac.trim().to_string(),
                ip,
                state: state.to_string(),
            });
        }
    }
    ifaces
}

fn uptime_info() -> Option<UptimeInfo> {
    let content = std::fs::read_to_string("/proc/uptime").ok()?;
    let seconds: f64 = content.split_whitespace().next()?.parse().ok()?;
    Some(UptimeInfo { seconds })
}

// ── Helpers ──

fn read_first_line(path: impl AsRef<Path>) -> Option<String> {
    std::fs::read_to_string(path)
        .ok()
        .map(|s| s.lines().next().unwrap_or("").to_string())
}

fn read_keyval_file(path: &str) -> std::collections::HashMap<String, String> {
    let mut map = std::collections::HashMap::new();
    let content = std::fs::read_to_string(path).ok();
    for line in content.iter().flat_map(|s| s.lines()) {
        if let Some(eq) = line.find('=') {
            let key = line[..eq].trim().to_string();
            let val = line[eq + 1..].trim().trim_matches('"').to_string();
            map.insert(key, val);
        }
    }
    map
}

/// Parse colon-separated key-value file (e.g. /proc/cpuinfo, /proc/meminfo).
/// Format: "key\t: value" or "key: value"
fn read_colon_file(path: &str) -> std::collections::HashMap<String, String> {
    let mut map = std::collections::HashMap::new();
    let content = std::fs::read_to_string(path).ok();
    for line in content.iter().flat_map(|s| s.lines()) {
        if let Some(colon) = line.find(':') {
            let key = line[..colon].trim().to_string();
            let val = line[colon + 1..].trim().to_string();
            // Only insert first occurrence (cpuinfo has per-core entries)
            map.entry(key).or_insert(val);
        }
    }
    map
}

fn parse_kb(s: &str) -> Option<u64> {
    s.split_whitespace().next().and_then(|n| n.parse().ok())
}

fn cmd_output(cmd: &str) -> Option<String> {
    cmd_output_opt(cmd).filter(|s| !s.is_empty())
}

fn cmd_output_opt(cmd: &str) -> Option<String> {
    let parts: Vec<&str> = cmd.split_whitespace().collect();
    if parts.is_empty() {
        return None;
    }
    std::process::Command::new(parts[0])
        .args(&parts[1..])
        .output()
        .ok()
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
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

fn find_mount(device: &str) -> Option<String> {
    std::fs::read_to_string("/proc/mounts").ok().and_then(|s| {
        for line in s.lines() {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 2
                && (parts[0].ends_with(device) || parts[0].contains(&format!("/{}", device)))
            {
                return Some(parts[1].to_string());
            }
        }
        // Check /proc/self/mountinfo for bind mounts
        std::fs::read_to_string("/proc/self/mountinfo")
            .ok()
            .and_then(|m| {
                for line in m.lines() {
                    let parts: Vec<&str> = line.split_whitespace().collect();
                    if parts.len() >= 5
                        && (parts[3].ends_with(device)
                            || parts[4].ends_with(&format!("/{}", device)))
                    {
                        return Some(parts[4].to_string());
                    }
                }
                None
            })
    })
}

fn find_ip(iface: &str) -> Option<String> {
    let output = std::process::Command::new("ip")
        .args(["-4", "addr", "show", iface])
        .output()
        .ok()?;
    let s = String::from_utf8_lossy(&output.stdout);
    for line in s.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("inet ") {
            let fields: Vec<&str> = trimmed.split_whitespace().collect();
            if fields.len() >= 2 {
                return Some(fields[1].split('/').next().unwrap_or(fields[1]).to_string());
            }
        }
    }
    None
}

fn pci_vendor_name(hex: &str) -> &'static str {
    match hex.trim_start_matches("0x").trim() {
        "8086" => "Intel",
        "10de" => "NVIDIA",
        "1002" => "AMD",
        "1a03" => "ASPEED",
        "1414" => "Microsoft",
        "1ae0" => "Google",
        "15ad" => "VMware",
        "80ee" => "Oracle",
        "1234" => "QEMU",
        _ => "Unknown",
    }
}
