use agnt_common::models::{ModelProfile, ModelsConfig};
use std::path::PathBuf;
use std::process::Command;

pub fn config_dir() -> PathBuf {
    std::env::var("AGNTOS_CONFIG_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("/etc/agntos"))
}

pub fn models_path() -> PathBuf {
    config_dir().join("models.toml")
}

pub fn api_key_for(profile: &ModelProfile) -> String {
    profile
        .api_key_env
        .as_ref()
        .and_then(|k| std::env::var(k).ok())
        .unwrap_or_else(|| "not-needed".into())
}

pub fn fetch_openai_models(endpoint: &str, api_key: &str) -> Result<Vec<(String, String)>, String> {
    let base = endpoint.trim_end_matches('/');
    let url = format!("{}/models", base);
    let mut cmd = Command::new("curl");
    cmd.args(["-sS", "--max-time", "20", "-H", "Accept: application/json"]);
    if !api_key.is_empty() && api_key != "not-needed" {
        cmd.args(["-H", &format!("Authorization: Bearer {}", api_key)]);
    }
    cmd.arg(&url);
    let output = cmd
        .output()
        .map_err(|e| format!("curl failed: {}", e))?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("GET {} failed: {}", url, stderr.trim()));
    }
    let body = String::from_utf8_lossy(&output.stdout);
    let parsed: serde_json::Value =
        serde_json::from_str(&body).map_err(|e| format!("Invalid JSON from {}: {}", url, e))?;
    let data = parsed
        .get("data")
        .and_then(|d| d.as_array())
        .ok_or_else(|| format!("No models list in response from {}", url))?;
    let mut models = Vec::new();
    for item in data {
        let id = item
            .get("id")
            .and_then(|v| v.as_str())
            .unwrap_or_default()
            .to_string();
        if id.is_empty() {
            continue;
        }
        let name = item
            .get("name")
            .and_then(|v| v.as_str())
            .unwrap_or(id.as_str())
            .to_string();
        models.push((id, name));
    }
    models.sort_by(|a, b| a.1.cmp(&b.1));
    Ok(models)
}

fn push_profile_models(
    name: &str,
    profile: &ModelProfile,
    options: &mut Vec<serde_json::Value>,
    models: &[(String, String)],
) -> Vec<serde_json::Value> {
    models
        .iter()
        .map(|(id, label)| {
            options.push(serde_json::json!({
                "provider": name,
                "modelId": id,
                "label": format!("{} · {}", name, label),
                "value": format!("{}/{}", name, id),
            }));
            serde_json::json!({ "id": id, "name": label })
        })
        .collect()
}

fn fallback_models(profile: &ModelProfile) -> Vec<(String, String)> {
    if profile.model.is_empty() {
        return Vec::new();
    }
    vec![(profile.model.clone(), profile.model.clone())]
}

pub fn initial_pi_model(cfg: &ModelsConfig) -> Option<String> {
    let (provider, profile) = cfg.chat_selection();
    if !profile.model.is_empty() {
        return Some(format!("{}/{}", provider, profile.model));
    }
    let api_key = api_key_for(profile);
    if let Ok(models) = fetch_openai_models(&profile.endpoint, &api_key) {
        if let Some((id, _)) = models.first() {
            return Some(format!("{}/{}", provider, id));
        }
    }
    let fallbacks = fallback_models(profile);
    fallbacks
        .first()
        .map(|(id, _)| format!("{}/{}", provider, id))
}

pub fn build_catalog(cfg: &ModelsConfig) -> serde_json::Value {
    let mut providers = Vec::new();
    let mut options = Vec::new();
    let (sel_provider, sel_profile) = cfg.chat_selection();
    let selected = if sel_profile.model.is_empty() {
        serde_json::Value::Null
    } else {
        serde_json::json!({
            "provider": sel_provider,
            "modelId": sel_profile.model,
            "value": format!("{}/{}", sel_provider, sel_profile.model),
        })
    };

    for (name, profile) in cfg.catalog_profiles() {
        let api_key = api_key_for(profile);
        let probe = fetch_openai_models(&profile.endpoint, &api_key);
        let models = match probe {
            Ok(list) if !list.is_empty() => push_profile_models(name, profile, &mut options, &list),
            Ok(_) => {
                let fallbacks = fallback_models(profile);
                if fallbacks.is_empty() {
                    providers.push(serde_json::json!({
                        "id": name,
                        "endpoint": profile.endpoint,
                        "error": "No models returned from endpoint",
                        "models": [],
                    }));
                    continue;
                }
                push_profile_models(name, profile, &mut options, &fallbacks)
            }
            Err(e) => {
                let fallbacks = fallback_models(profile);
                if !fallbacks.is_empty() {
                    let entries =
                        push_profile_models(name, profile, &mut options, &fallbacks);
                    providers.push(serde_json::json!({
                        "id": name,
                        "endpoint": profile.endpoint,
                        "error": e,
                        "models": entries,
                    }));
                    continue;
                }
                providers.push(serde_json::json!({
                    "id": name,
                    "endpoint": profile.endpoint,
                    "error": e,
                    "models": [],
                }));
                continue;
            }
        };
        providers.push(serde_json::json!({
            "id": name,
            "endpoint": profile.endpoint,
            "models": models,
        }));
    }

    serde_json::json!({
        "providers": providers,
        "options": options,
        "selected": selected,
    })
}

pub fn write_pi_models_json(cfg: &ModelsConfig, agent_dir: &std::path::Path) -> Result<(), String> {
    let mut providers = serde_json::Map::new();

    for (name, profile) in cfg.catalog_profiles() {
        let api_key = api_key_for(profile);
        let models = fetch_openai_models(&profile.endpoint, &api_key).unwrap_or_default();
        if models.is_empty() && !profile.model.is_empty() {
            providers.insert(
                name.to_string(),
                serde_json::json!({
                    "baseUrl": profile.endpoint,
                    "api": "openai-completions",
                    "apiKey": api_key,
                    "models": [{
                        "id": profile.model,
                        "name": profile.model,
                        "reasoning": true,
                        "input": ["text"],
                        "contextWindow": 131072,
                        "maxTokens": profile.max_tokens,
                        "cost": { "input": 0, "output": 0, "cacheRead": 0, "cacheWrite": 0 }
                    }]
                }),
            );
            continue;
        }
        let model_entries: Vec<_> = models
            .into_iter()
            .map(|(id, label)| {
                serde_json::json!({
                    "id": id,
                    "name": label,
                    "reasoning": true,
                    "input": ["text"],
                    "contextWindow": 131072,
                    "maxTokens": profile.max_tokens,
                    "cost": { "input": 0, "output": 0, "cacheRead": 0, "cacheWrite": 0 }
                })
            })
            .collect();
        if model_entries.is_empty() {
            continue;
        }
        providers.insert(
            name.to_string(),
            serde_json::json!({
                "baseUrl": profile.endpoint,
                "api": "openai-completions",
                "apiKey": api_key,
                "models": model_entries,
            }),
        );
    }

    if providers.is_empty() {
        return Ok(());
    }

    let payload = serde_json::json!({ "providers": providers });
    std::fs::write(
        agent_dir.join("models.json"),
        serde_json::to_string_pretty(&payload).map_err(|e| e.to_string())?,
    )
    .map_err(|e| format!("Failed to write Pi models.json: {}", e))?;
    Ok(())
}
