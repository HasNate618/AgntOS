use serde::{Deserialize, Serialize};
use std::path::Path;

use crate::paths;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ApplyPolicy {
    Manual,
    Auto,
}

impl Default for ApplyPolicy {
    fn default() -> Self {
        Self::Manual
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgntosSettings {
    #[serde(default)]
    pub auto_apply: ApplyPolicy,
}

impl Default for AgntosSettings {
    fn default() -> Self {
        Self {
            auto_apply: ApplyPolicy::Manual,
        }
    }
}

impl AgntosSettings {
    pub fn load_from_config_dir(config_dir: impl AsRef<Path>) -> Self {
        let path = config_dir.as_ref().join("settings.json");
        if !path.exists() {
            return Self::default();
        }
        let raw = std::fs::read_to_string(&path).unwrap_or_default();
        serde_json::from_str(&raw).unwrap_or_default()
    }

    pub fn load() -> Self {
        Self::load_from_config_dir(paths::nix_config_dir())
    }
}
