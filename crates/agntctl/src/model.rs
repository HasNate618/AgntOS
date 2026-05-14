//! `agntctl model` — lists configured model profiles and resolves task routes
//! from `/etc/agntos/models.toml`.

use agnt_common::models::ModelsConfig;
use std::path::PathBuf;

const DEFAULT_CONFIG_DIR: &str = "/etc/agntos";

fn models_path(config_dir: Option<&PathBuf>) -> PathBuf {
    config_dir
        .cloned()
        .unwrap_or_else(|| PathBuf::from(DEFAULT_CONFIG_DIR))
        .join("models.toml")
}

pub fn execute_list(json: bool, config_dir: Option<&PathBuf>) -> Result<String, String> {
    let path = models_path(config_dir);
    let cfg = ModelsConfig::from_path(&path)?;

    if json {
        return serde_json::to_string_pretty(&cfg)
            .map_err(|e| format!("Failed to serialize model config: {}", e));
    }

    let mut out = String::new();
    out.push_str(&format!("Model profiles ({})\n\n", path.display()));
    for (name, profile) in cfg.named_profiles() {
        out.push_str(&format!(
            "  - {:<12} model={} endpoint={} max_tokens={} temp={}\n",
            name, profile.model, profile.endpoint, profile.max_tokens, profile.temperature
        ));
        if let Some(api_key_env) = &profile.api_key_env {
            out.push_str(&format!("    api_key_env={}\n", api_key_env));
        }
    }

    if cfg.routing.is_empty() {
        out.push_str("\nNo explicit task routing configured. All tasks use default.\n");
        return Ok(out);
    }

    let mut routes: Vec<_> = cfg.routing.iter().collect();
    routes.sort_by(|a, b| a.0.cmp(b.0));
    out.push_str("\nTask routing\n");
    for (task, profile_name) in routes {
        if let Some((resolved_name, profile)) = cfg.profile_for_task(task) {
            out.push_str(&format!(
                "  - {:<12} -> {:<12} ({})\n",
                task, resolved_name, profile.model
            ));
        } else {
            out.push_str(&format!(
                "  - {:<12} -> {:<12} (invalid profile)\n",
                task, profile_name
            ));
        }
    }

    Ok(out)
}

pub fn execute_route(
    task: &str,
    json: bool,
    config_dir: Option<&PathBuf>,
) -> Result<String, String> {
    let path = models_path(config_dir);
    let cfg = ModelsConfig::from_path(&path)?;

    let (profile_name, profile) = cfg.profile_for_task(task).ok_or_else(|| {
        format!(
            "Task '{}' routes to an unknown profile in {}",
            task,
            path.display()
        )
    })?;

    if json {
        let payload = serde_json::json!({
            "task": task,
            "profile": profile_name,
            "endpoint": profile.endpoint,
            "model": profile.model,
            "api_key_env": profile.api_key_env,
            "max_tokens": profile.max_tokens,
            "temperature": profile.temperature,
        });
        return serde_json::to_string_pretty(&payload)
            .map_err(|e| format!("Failed to serialize route: {}", e));
    }

    Ok(format!(
        "Task:      {}\nProfile:   {}\nModel:     {}\nEndpoint:  {}\nMax tokens:{}\nTemp:      {}\n",
        task,
        profile_name,
        profile.model,
        profile.endpoint,
        profile.max_tokens,
        profile.temperature,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn write_models(dir: &PathBuf) {
        std::fs::create_dir_all(dir).unwrap();
        std::fs::write(
            dir.join("models.toml"),
            r#"
[default]
endpoint = "https://api.example.com/v1"
model = "gpt-4o-mini"
api_key_env = "AGNTOS_API_KEY"
max_tokens = 4096
temperature = 0.2

[local]
endpoint = "http://127.0.0.1:11434/v1"
model = "qwen2.5-coder:7b"
max_tokens = 2048
temperature = 0.1

[routing]
inspect = "local"
apply = "default"
"#,
        )
        .unwrap();
    }

    #[test]
    fn list_works() {
        let dir = std::env::temp_dir().join("agntctl-model-list-test");
        let _ = std::fs::remove_dir_all(&dir);
        write_models(&dir);

        let out = execute_list(false, Some(&dir)).unwrap();
        assert!(out.contains("Task routing"));
        assert!(out.contains("inspect"));

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn route_fallbacks_to_default() {
        let dir = std::env::temp_dir().join("agntctl-model-route-test");
        let _ = std::fs::remove_dir_all(&dir);
        write_models(&dir);

        let out = execute_route("unknown-task", false, Some(&dir)).unwrap();
        assert!(out.contains("Profile:   default"));

        let _ = std::fs::remove_dir_all(&dir);
    }
}
