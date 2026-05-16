#[derive(Debug, Clone)]
pub struct StatusModel {
    pub connected: bool,
    pub profile_name: String,
    pub model_name: String,
    pub endpoint: String,
    pub cpu_info: String,
    pub ram_used: String,
    pub disk_used: String,
    pub failed_units: u32,
    pub watchdog_interval: u64,
    pub watchdog_disk_threshold: u8,
    pub watchdog_alert_count: u32,
    pub last_check_time: String,
}

impl StatusModel {
    pub fn new() -> Self {
        Self {
            connected: false,
            profile_name: String::new(),
            model_name: String::new(),
            endpoint: String::new(),
            cpu_info: String::new(),
            ram_used: String::new(),
            disk_used: String::new(),
            failed_units: 0,
            watchdog_interval: 300,
            watchdog_disk_threshold: 95,
            watchdog_alert_count: 0,
            last_check_time: String::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn status_model_defaults() {
        let s = StatusModel::new();
        assert!(!s.connected);
        assert_eq!(s.watchdog_interval, 300);
        assert_eq!(s.watchdog_disk_threshold, 95);
    }

    #[test]
    fn status_model_updates() {
        let mut s = StatusModel::new();
        s.connected = true;
        s.profile_name = "local".to_string();
        s.model_name = "qwen".to_string();
        s.cpu_info = "8 cores".to_string();
        s.ram_used = "4.2 / 32 GB".to_string();
        s.disk_used = "45%".to_string();
        assert_eq!(s.profile_name, "local");
        assert_eq!(s.cpu_info, "8 cores");
    }
}
