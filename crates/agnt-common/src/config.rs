use serde::{Deserialize, Serialize};

/// AgntOS-managed Nix config tree.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgntosConfig {
    pub config_dir: String,
    pub profiles: Vec<String>,
    pub packages: Vec<String>,
    pub services: Vec<String>,
    pub options: std::collections::HashMap<String, ConfigValue>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ConfigValue {
    String(String),
    Number(f64),
    Bool(bool),
    List(Vec<ConfigValue>),
    Map(std::collections::HashMap<String, ConfigValue>),
}

impl AgntosConfig {
    pub fn new(config_dir: &str) -> Self {
        Self {
            config_dir: config_dir.to_string(),
            profiles: Vec::new(),
            packages: Vec::new(),
            services: Vec::new(),
            options: std::collections::HashMap::new(),
        }
    }
}

/// A proposed change to the AgntOS config tree.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigProposal {
    pub id: String,
    pub summary: String,
    pub nix_changes: String,
    pub files_to_write: Vec<(String, String)>,
    pub files_to_delete: Vec<String>,
    pub rollback_guidance: String,
}
