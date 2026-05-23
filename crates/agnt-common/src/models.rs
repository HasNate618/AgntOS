//! Model-routing configuration types.
//!
//! Parses `/etc/agntos/models.toml` into [`ModelsConfig`]. Providers hold
//! OpenAI-compatible endpoint URLs; the chat UI selects a concrete model id
//! from each provider's `/v1/models` listing.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelsConfig {
    pub default: ModelProfile,
    #[serde(default)]
    pub profiles: HashMap<String, ModelProfile>,
    #[serde(default)]
    pub routing: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelProfile {
    pub endpoint: String,
    #[serde(default)]
    pub model: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub api_key_env: Option<String>,
    #[serde(default = "default_max_tokens")]
    pub max_tokens: u32,
    #[serde(default = "default_temperature")]
    pub temperature: f32,
    #[serde(default = "default_supports_tools")]
    pub supports_tools: bool,
}

fn default_supports_tools() -> bool {
    true
}

fn default_max_tokens() -> u32 {
    4096
}

fn default_temperature() -> f32 {
    0.7
}

impl ModelProfile {
    fn from_toml_table(table: &toml::value::Table) -> Result<Self, String> {
        let endpoint = table
            .get("endpoint")
            .and_then(|v| v.as_str())
            .ok_or("profile missing endpoint")?
            .to_string();
        let model = table
            .get("model")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        let api_key_env = table
            .get("api_key_env")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());
        let max_tokens = table
            .get("max_tokens")
            .and_then(|v| v.as_integer())
            .map(|v| v as u32)
            .unwrap_or_else(default_max_tokens);
        let temperature = table
            .get("temperature")
            .and_then(|v| v.as_float())
            .map(|v| v as f32)
            .unwrap_or_else(default_temperature);
        let supports_tools = table
            .get("supports_tools")
            .and_then(|v| v.as_bool())
            .unwrap_or_else(default_supports_tools);
        Ok(Self {
            endpoint,
            model,
            api_key_env,
            max_tokens,
            temperature,
            supports_tools,
        })
    }
}

impl ModelsConfig {
    pub fn from_toml_str(input: &str) -> Result<Self, String> {
        let root: toml::Value =
            toml::from_str(input).map_err(|e| format!("Failed to parse models.toml: {}", e))?;
        let table = root
            .as_table()
            .ok_or("models.toml root must be a table")?;

        let default_table = table
            .get("default")
            .and_then(|v| v.as_table())
            .ok_or("models.toml missing [default] section")?;
        let default = ModelProfile::from_toml_table(default_table)?;

        let mut routing = HashMap::new();
        if let Some(route_table) = table.get("routing").and_then(|v| v.as_table()) {
            for (k, v) in route_table {
                if let Some(s) = v.as_str() {
                    routing.insert(k.clone(), s.to_string());
                }
            }
        }

        let mut profiles = HashMap::new();
        if let Some(nested) = table.get("profiles").and_then(|v| v.as_table()) {
            for (name, value) in nested {
                if let Some(t) = value.as_table() {
                    profiles.insert(name.clone(), ModelProfile::from_toml_table(t)?);
                }
            }
        }

        for (key, value) in table {
            if matches!(key.as_str(), "default" | "routing" | "profiles" | "pi" | "agntos" | "host" | "port") {
                continue;
            }
            if let Some(t) = value.as_table() {
                if t.contains_key("endpoint") {
                    profiles.insert(key.clone(), ModelProfile::from_toml_table(t)?);
                }
            }
        }

        Ok(Self {
            default,
            profiles,
            routing,
        })
    }

    pub fn from_path(path: impl AsRef<Path>) -> Result<Self, String> {
        let content = std::fs::read_to_string(path.as_ref())
            .map_err(|e| format!("Failed to read {}: {}", path.as_ref().display(), e))?;
        Self::from_toml_str(&content)
    }

    pub fn to_toml_string(&self) -> Result<String, String> {
        #[derive(Serialize)]
        struct Out<'a> {
            default: &'a ModelProfile,
            #[serde(skip_serializing_if = "HashMap::is_empty")]
            profiles: &'a HashMap<String, ModelProfile>,
            #[serde(skip_serializing_if = "HashMap::is_empty")]
            routing: &'a HashMap<String, String>,
        }
        toml::to_string_pretty(&Out {
            default: &self.default,
            profiles: &self.profiles,
            routing: &self.routing,
        })
        .map_err(|e| format!("Failed to serialize models.toml: {}", e))
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

    pub fn catalog_profiles(&self) -> Vec<(&str, &ModelProfile)> {
        if !self.profiles.is_empty() {
            let mut out: Vec<(&str, &ModelProfile)> =
                self.profiles.iter().map(|(k, v)| (k.as_str(), v)).collect();
            out.sort_by(|a, b| a.0.cmp(b.0));
            return out;
        }
        if let Some((name, profile)) = self.profile_for_task("chat") {
            return vec![(name, profile)];
        }
        vec![("default", &self.default)]
    }

    pub fn chat_selection(&self) -> (&str, &ModelProfile) {
        self.profile_for_task("chat")
            .unwrap_or(("default", &self.default))
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

[fast]
endpoint = "https://local.example/v1"
model = "qwen2.5-coder:7b"

[routing]
inspect = "fast"
apply = "default"
"#;

    #[test]
    fn parse_legacy_flat_profiles_and_routing() {
        let cfg = ModelsConfig::from_toml_str(SAMPLE).unwrap();
        assert_eq!(cfg.default.model, "gpt-4o-mini");
        assert_eq!(cfg.routing.get("inspect").unwrap(), "fast");
        assert!(cfg.profiles.contains_key("fast"));
    }

    #[test]
    fn parse_nested_profiles_table() {
        let input = r#"
[default]
endpoint = "https://api.example.com/v1"
model = "gpt-4o-mini"

[profiles.fast]
endpoint = "https://local.example/v1"
model = "qwen2.5-coder:7b"

[routing]
inspect = "fast"
"#;
        let cfg = ModelsConfig::from_toml_str(input).unwrap();
        assert!(cfg.profiles.contains_key("fast"));
        assert_eq!(cfg.routing.get("inspect").unwrap(), "fast");
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

    #[test]
    fn roundtrip_serialize_keeps_routing_separate() {
        let cfg = ModelsConfig::from_toml_str(SAMPLE).unwrap();
        let out = cfg.to_toml_string().unwrap();
        let again = ModelsConfig::from_toml_str(&out).unwrap();
        assert_eq!(again.routing.get("inspect").unwrap(), "fast");
        assert!(again.profiles.contains_key("fast"));
    }

    #[test]
    fn catalog_profiles_skips_default_when_profiles_exist() {
        let input = r#"
[default]
endpoint = "https://api.example.com/v1"
model = "gpt-4o-mini"

[profiles.gateway]
endpoint = "http://10.0.0.45/bifrost/v1"
model = "test-model"

[routing]
chat = "gateway"
"#;
        let cfg = ModelsConfig::from_toml_str(input).unwrap();
        let names: Vec<_> = cfg.catalog_profiles().into_iter().map(|(n, _)| n).collect();
        assert_eq!(names, vec!["gateway"]);
        let (p, prof) = cfg.chat_selection();
        assert_eq!(p, "gateway");
        assert_eq!(prof.model, "test-model");
    }
}
