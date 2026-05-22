use crate::inspect;
use agnt_common::models::{ModelProfile, ModelsConfig};
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

pub fn execute_add(
    name: &str,
    endpoint: &str,
    model: Option<&str>,
    api_key_env: Option<&str>,
    max_tokens: Option<u32>,
    temperature: Option<f32>,
    config_dir: Option<&PathBuf>,
) -> Result<String, String> {
    let path = models_path(config_dir);
    let mut cfg = ModelsConfig::from_path(&path)?;

    if name == "default" {
        return Err("Use 'model set-default' to change default profile settings".to_string());
    }

    if cfg.profiles.contains_key(name) {
        return Err(format!(
            "Profile '{}' already exists. Remove it first or use a different name.",
            name
        ));
    }

    let profile = ModelProfile {
        endpoint: endpoint.to_string(),
        model: model.unwrap_or("").to_string(),
        api_key_env: api_key_env.map(|s| s.to_string()),
        max_tokens: max_tokens.unwrap_or(4096),
        temperature: temperature.unwrap_or(0.7),
    };

    cfg.profiles.insert(name.to_string(), profile);

    let toml_str = cfg
        .to_toml_string()
        .map_err(|e| format!("Failed to serialize config: {}", e))?;

    std::fs::write(&path, &toml_str)
        .map_err(|e| format!("Failed to write {}: {}", path.display(), e))?;

    let model_note = if model.unwrap_or("").is_empty() {
        "(pick in chat)"
    } else {
        model.unwrap_or("")
    };
    Ok(format!(
        "Profile '{}' added (endpoint={}, model={})",
        name, endpoint, model_note
    ))
}

pub fn execute_remove(name: &str, config_dir: Option<&PathBuf>) -> Result<String, String> {
    let path = models_path(config_dir);
    let mut cfg = ModelsConfig::from_path(&path)?;

    if name == "default" {
        return Err("Cannot remove 'default' profile".to_string());
    }

    if !cfg.profiles.contains_key(name) {
        return Err(format!("Profile '{}' not found", name));
    }

    cfg.profiles.remove(name);
    cfg.routing.retain(|_, v| v != name);

    let toml_str = cfg
        .to_toml_string()
        .map_err(|e| format!("Failed to serialize config: {}", e))?;

    std::fs::write(&path, &toml_str)
        .map_err(|e| format!("Failed to write {}: {}", path.display(), e))?;

    Ok(format!("Profile '{}' removed", name))
}

pub fn execute_set_route(
    task: &str,
    profile: &str,
    config_dir: Option<&PathBuf>,
) -> Result<String, String> {
    let path = models_path(config_dir);
    let mut cfg = ModelsConfig::from_path(&path)?;

    if profile != "default" && !cfg.profiles.contains_key(profile) {
        let available: Vec<&str> = cfg.named_profiles().iter().map(|(n, _)| *n).collect();
        return Err(format!(
            "Profile '{}' not found. Available: {}",
            profile,
            available.join(", ")
        ));
    }

    cfg.routing.insert(task.to_string(), profile.to_string());

    let toml_str = cfg
        .to_toml_string()
        .map_err(|e| format!("Failed to serialize config: {}", e))?;

    std::fs::write(&path, &toml_str)
        .map_err(|e| format!("Failed to write {}: {}", path.display(), e))?;

    Ok(format!("Task '{}' routed to profile '{}'", task, profile))
}

pub fn execute_suggest(config_dir: Option<&PathBuf>) -> Result<String, String> {
    let info = inspect::SystemInfo::collect();

    let mem_mb = info.memory.total_kb / 1024;
    let cpu_count = info.cpu.cores;
    let cpu_model = &info.cpu.model;
    let has_gpu = !info.gpu.is_empty();

    let (rec_model, rec_params, rec_notes) = if mem_mb < 4000 {
        (
            "qwen2.5-coder:1.5b",
            "1.5B params",
            "Very limited RAM — use small quantized models or cloud endpoints.",
        )
    } else if mem_mb < 8000 {
        (
            "qwen2.5-coder:7b",
            "7B params",
            "8GB RAM can run quantized 7B-14B models locally. Ollama recommended.",
        )
    } else if mem_mb < 16000 {
        (
            "qwen2.5-coder:14b",
            "14B params",
            "16GB RAM can run 14B-32B models with reasonable speed.",
        )
    } else if mem_mb < 32000 {
        (
            "qwen2.5-coder:32b",
            "32B params",
            "32GB+ RAM can run larger models. Consider GPU offloading.",
        )
    } else {
        (
            "qwen2.5-coder:32b",
            "32B params",
            "Ample RAM. Consider larger models with GPU acceleration.",
        )
    };

    let gpu_note = if has_gpu {
        let models: Vec<&str> = info
            .gpu
            .iter()
            .filter_map(|g| {
                if !g.model.is_empty() {
                    Some(g.model.as_str())
                } else {
                    None
                }
            })
            .collect();
        format!(
            "Detected GPU(s): {}. GPU offloading available for llama.cpp / Ollama.",
            models.join(", ")
        )
    } else {
        "No GPU detected. CPU-only inference via Ollama or llama.cpp.".to_string()
    };

    let path = models_path(config_dir);
    let example_config = format!(
        "\nExample config (add to {}):\n\
         ---\n\
         [default]\n\
         endpoint = \"http://localhost:11434/v1\"\n\
         model = \"{}\"\n\
         api_key_env = \"AGNTOS_API_KEY\"\n\
         max_tokens = 4096\n\
         temperature = 0.2\n\
         \n\
         [fast]\n\
         endpoint = \"http://localhost:11434/v1\"\n\
         model = \"qwen2.5-coder:1.5b\"\n\
         max_tokens = 2048\n\
         temperature = 0.1\n\
         \n\
         [routing]\n\
         inspect = \"fast\"\n\
         chat = \"default\"\n\
         ---\n",
        path.display(),
        rec_model
    );

    Ok(format!(
        "System hardware:\n\
         RAM:     {} MB\n\
         CPU:     {} ({} cores)\n\
         GPU:     {}\n\
         {}\n\
         \n\
         Recommended local model: {} ({})\n\
         {}\n\
         {}",
        mem_mb,
        cpu_model,
        cpu_count,
        if has_gpu { "detected" } else { "none" },
        gpu_note,
        rec_model,
        rec_params,
        rec_notes,
        example_config
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

    fn empty_models(dir: &PathBuf) {
        std::fs::create_dir_all(dir).unwrap();
        std::fs::write(
            dir.join("models.toml"),
            r#"
[default]
endpoint = "http://localhost:11434/v1"
model = "qwen2.5-coder:7b"
max_tokens = 4096
temperature = 0.7
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

    #[test]
    fn add_profile() {
        let dir = std::env::temp_dir().join("agntctl-model-add-test");
        let _ = std::fs::remove_dir_all(&dir);
        empty_models(&dir);

        let out = execute_add(
            "fast",
            "http://127.0.0.1:11434/v1",
            Some("tiny-model"),
            None,
            None,
            None,
            Some(&dir),
        )
        .unwrap();
        assert!(out.contains("Profile 'fast' added"));

        let cfg = ModelsConfig::from_path(&dir.join("models.toml")).unwrap();
        assert!(cfg.profiles.contains_key("fast"));

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn remove_profile() {
        let dir = std::env::temp_dir().join("agntctl-model-remove-test");
        let _ = std::fs::remove_dir_all(&dir);
        write_models(&dir);

        let out = execute_remove("local", Some(&dir)).unwrap();
        assert!(out.contains("Profile 'local' removed"));

        let cfg = ModelsConfig::from_path(&dir.join("models.toml")).unwrap();
        assert!(!cfg.profiles.contains_key("local"));

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn cannot_remove_default() {
        let dir = std::env::temp_dir().join("agntctl-model-remove-default-test");
        let _ = std::fs::remove_dir_all(&dir);
        empty_models(&dir);

        let err = execute_remove("default", Some(&dir)).unwrap_err();
        assert!(err.contains("Cannot remove"));

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn set_route() {
        let dir = std::env::temp_dir().join("agntctl-model-route-set-test");
        let _ = std::fs::remove_dir_all(&dir);
        write_models(&dir);

        let out = execute_set_route("memory", "local", Some(&dir)).unwrap();
        assert!(out.contains("memory"));

        let cfg = ModelsConfig::from_path(&dir.join("models.toml")).unwrap();
        assert_eq!(cfg.routing.get("memory").map(|s| s.as_str()), Some("local"));

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn set_route_invalid_profile() {
        let dir = std::env::temp_dir().join("agntctl-model-route-invalid-test");
        let _ = std::fs::remove_dir_all(&dir);
        empty_models(&dir);

        let err = execute_set_route("chat", "nonexistent", Some(&dir)).unwrap_err();
        assert!(err.contains("nonexistent"));

        let _ = std::fs::remove_dir_all(&dir);
    }
}
