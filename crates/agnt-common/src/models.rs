use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelsConfig {
    pub default: ModelProfile,
    #[serde(default)]
    pub routing: HashMap<String, String>,
    #[serde(flatten)]
    pub profiles: HashMap<String, ModelProfile>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelProfile {
    pub endpoint: String,
    pub model: String,
    pub api_key_env: Option<String>,
    pub max_tokens: u32,
    pub temperature: f32,
}

impl ModelsConfig {
    pub fn from_toml_str(input: &str) -> Result<Self, String> {
        toml::from_str::<Self>(input).map_err(|e| format!("Failed to parse models.toml: {}", e))
    }

    pub fn from_path(path: impl AsRef<Path>) -> Result<Self, String> {
        let content = std::fs::read_to_string(path.as_ref())
            .map_err(|e| format!("Failed to read {}: {}", path.as_ref().display(), e))?;
        Self::from_toml_str(&content)
    }

    pub fn profile_for_task(&self, task: &str) -> Option<(&str, &ModelProfile)> {
        if let Some(profile_name) = self.routing.get(task) {
            if profile_name == "default" {
                return Some(("default", &self.default));
            }
            if let Some(profile) = self.profiles.get(profile_name) {
                return Some((profile_name.as_str(), profile));
            }
            return None;
        }
        Some(("default", &self.default))
    }

    pub fn named_profiles(&self) -> Vec<(&str, &ModelProfile)> {
        let mut out = vec![("default", &self.default)];
        let mut extra: Vec<(&str, &ModelProfile)> =
            self.profiles.iter().map(|(k, v)| (k.as_str(), v)).collect();
        extra.sort_by(|a, b| a.0.cmp(b.0));
        out.extend(extra);
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: &str = r#"
[default]
endpoint = "https://api.example.com/v1"
model = "gpt-4o-mini"
api_key_env = "AGNTOS_API_KEY"
max_tokens = 4096
temperature = 0.2

[fast]
endpoint = "https://local.example/v1"
model = "qwen2.5-coder:7b"
max_tokens = 2048
temperature = 0.4

[routing]
inspect = "fast"
apply = "default"
"#;

    #[test]
    fn parse_models_toml() {
        let cfg = ModelsConfig::from_toml_str(SAMPLE).unwrap();
        assert_eq!(cfg.default.model, "gpt-4o-mini");
        assert_eq!(cfg.routing.get("inspect").unwrap(), "fast");
        assert!(cfg.profiles.contains_key("fast"));
    }

    #[test]
    fn resolve_task_route() {
        let cfg = ModelsConfig::from_toml_str(SAMPLE).unwrap();
        let (name, profile) = cfg.profile_for_task("inspect").unwrap();
        assert_eq!(name, "fast");
        assert_eq!(profile.model, "qwen2.5-coder:7b");

        let (fallback_name, fallback_profile) = cfg.profile_for_task("unknown").unwrap();
        assert_eq!(fallback_name, "default");
        assert_eq!(fallback_profile.model, "gpt-4o-mini");
    }
}
