use serde::{Deserialize, Serialize};
use std::path::PathBuf;

fn config_dir() -> PathBuf {
    if let Ok(dir) = std::env::var("AGNTOS_CONFIG_DIR") {
        return PathBuf::from(dir);
    }
    if let Ok(home) = std::env::var("HOME") {
        let local = PathBuf::from(&home).join(".config/agntos");
        if local.join("AGENTS.md").exists() {
            return local;
        }
    }
    PathBuf::from("/etc/agntos")
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub config_dir: PathBuf,
    pub pi_binary: String,
    pub system_prompt_path: PathBuf,
    pub extension_path_dir: PathBuf,
    pub extension_path: PathBuf,
    pub model_config_path: PathBuf,
    pub agntctl_path: String,
    pub default_model: Option<String>,
    pub llm_base_url: String,
    pub host: String,
    pub port: u16,
}

impl Default for AppConfig {
    fn default() -> Self {
        let base = config_dir();
        Self {
            config_dir: base.clone(),
            pi_binary: "pi".into(),
            system_prompt_path: base.join("AGENTS.md"),
            extension_path_dir: base.join("extensions/agntos-tools"),
            extension_path: base.join("extensions/agntos-tools/index.ts"),
            model_config_path: base.join("models.toml"),
            agntctl_path: "agntctl".into(),
            default_model: None,
            llm_base_url: String::new(),
            host: "0.0.0.0".into(),
            port: 8080,
        }
    }
}

fn resolve_pi_binary() -> String {
    if let Ok(path) = std::env::var("AGNTOS_PI_BINARY") {
        if !path.is_empty() {
            return path;
        }
    }
    if let Ok(paths) = std::env::var("PATH") {
        for dir in paths.split(':') {
            let candidate = std::path::Path::new(dir).join("pi");
            if candidate.exists() {
                return candidate.to_string_lossy().into_owned();
            }
        }
    }
    for candidate in &[
        "/run/current-system/sw/bin/pi",
        "/usr/local/bin/pi",
        "/home/developer/.local/bin/pi",
    ] {
        if std::path::Path::new(candidate).exists() {
            return (*candidate).into();
        }
    }
    "pi".into()
}

impl AppConfig {
    pub fn load() -> Result<Self, Box<dyn std::error::Error>> {
        let base = config_dir();
        let config_path = base.join("models.toml");
        let mut config = Self::default();
        config.pi_binary = resolve_pi_binary();

        if config_path.exists() {
            let content = std::fs::read_to_string(&config_path)?;
            let parsed: toml::Value = content.parse::<toml::Value>()?;

            if let Ok(cfg) = agnt_common::models::ModelsConfig::from_toml_str(&content) {
                if !cfg.default.model.is_empty() {
                    config.default_model = Some(format!("default/{}", cfg.default.model));
                }
            }

            if let Some(pi) = parsed.get("pi") {
                if let Some(binary) = pi.get("binary").and_then(|v| v.as_str()) {
                    config.pi_binary = binary.into();
                }
            }

            if let Some(agntos) = parsed.get("agntos") {
                if let Some(prompt) = agntos.get("system_prompt").and_then(|v| v.as_str()) {
                    config.system_prompt_path = PathBuf::from(prompt);
                }
                if let Some(ext) = agntos.get("extension").and_then(|v| v.as_str()) {
                    config.extension_path = PathBuf::from(ext);
                }
                if let Some(model) = agntos.get("default_model").and_then(|v| v.as_str()) {
                    config.default_model = Some(model.into());
                }
                if let Some(url) = agntos.get("llm_base_url").and_then(|v| v.as_str()) {
                    config.llm_base_url = url.into();
                }
                if let Some(actl) = agntos.get("agntctl_path").and_then(|v| v.as_str()) {
                    config.agntctl_path = actl.into();
                }
            }

            if let Some(host) = parsed.get("host").and_then(|v| v.as_str()) {
                config.host = host.into();
            }
            if let Some(port) = parsed.get("port").and_then(|v| v.as_integer()) {
                config.port = port as u16;
            }
        }

        Ok(config)
    }
}
