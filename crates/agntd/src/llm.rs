//! OpenAI-compatible LLM client and tool-schema definitions.
//!
//! [`LlmClient`] connects to a user-configured endpoint (defined in
//! `/etc/agntos/models.toml`), sends chat-completion requests with AgntOS
//! tool definitions, and returns parsed assistant responses including
//! optional tool-call requests.
//!
//! [`tool_definitions`] returns the OpenAI function-calling schema for the
//! five AgntOS tools: `inspect`, `propose`, `apply`, `audit`, `memory`.
//!
//! [`build_system_prompt`] assembles the frozen memory snapshot, system
//! profile, and behavioural rules into a single system-level message.

use agnt_common::memory::{CoreMemory, MemoryFile};
use agnt_common::models::{ModelProfile, ModelsConfig};
use reqwest::header::{AUTHORIZATION, CONTENT_TYPE};
use serde::Deserialize;
use serde_json::{json, Value};
use std::path::Path;

#[derive(Clone)]
pub struct LlmClient {
    http: reqwest::Client,
    pub profile_name: String,
    pub profile: ModelProfile,
}

#[derive(Debug, Clone)]
pub struct AssistantResponse {
    pub assistant_message: Value,
    pub content: String,
    pub tool_calls: Vec<ToolCall>,
}

#[derive(Debug, Clone)]
pub struct ToolCall {
    pub id: String,
    pub name: String,
    pub arguments: Value,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct ChatResponse {
    choices: Vec<Choice>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct Choice {
    message: AssistantMessage,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct AssistantMessage {
    role: String,
    #[serde(default)]
    content: Option<String>,
    #[serde(default)]
    tool_calls: Option<Vec<RawToolCall>>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct RawToolCall {
    id: String,
    #[serde(rename = "type")]
    _kind: String,
    function: RawFunctionCall,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct RawFunctionCall {
    name: String,
    arguments: String,
}

impl LlmClient {
    pub fn from_config(config_dir: impl AsRef<Path>, task: &str) -> Result<Self, String> {
        let models_path = config_dir.as_ref().join("models.toml");
        let cfg = ModelsConfig::from_path(&models_path).map_err(|e| {
            format!(
                "{}\nHint: create {} (example at /etc/agntos/models.toml.example)",
                e,
                models_path.display()
            )
        })?;

        let (profile_name, profile) = cfg
            .profile_for_task(task)
            .ok_or_else(|| format!("No model route found for task '{}'", task))?;

        Ok(Self {
            http: reqwest::Client::new(),
            profile_name: profile_name.to_string(),
            profile: profile.clone(),
        })
    }

    /// Non-streaming completion (kept as fallback).
    #[allow(dead_code)]
    pub async fn complete(
        &self,
        messages: &[Value],
        tools: &[Value],
    ) -> Result<AssistantResponse, String> {
        let endpoint = normalize_endpoint(&self.profile.endpoint);
        let payload = completion_payload(&self.profile, messages, tools, false);
        let body = self.post_json(&endpoint, &payload).await?;
        parse_completion_response(&body)
    }

    /// Streaming completion: prints content deltas as they arrive, then
    /// returns the fully-assembled response (including any tool calls).
    pub async fn complete_streaming(
        &self,
        messages: &[Value],
        tools: &[Value],
    ) -> Result<AssistantResponse, String> {
        use futures_util::StreamExt;

        let endpoint = normalize_endpoint(&self.profile.endpoint);
        let payload = completion_payload(&self.profile, messages, tools, true);

        let mut req = self
            .http
            .post(&endpoint)
            .header(CONTENT_TYPE, "application/json")
            .json(&payload);

        if let Some(env_name) = &self.profile.api_key_env {
            if let Ok(key) = std::env::var(env_name) {
                if !key.trim().is_empty() {
                    req = req.header(AUTHORIZATION, format!("Bearer {}", key));
                }
            }
        }

        let resp = req
            .send()
            .await
            .map_err(|e| format!("LLM request failed: {}", e))?;
        let status = resp.status();
        if !status.is_success() {
            let body = resp.text().await.unwrap_or_default();
            return Err(format!("LLM returned {}: {}", status, body));
        }

        let mut stream = resp.bytes_stream();
        let mut content = String::new();
        let mut tool_call_buf: Vec<StreamToolCall> = Vec::new();

        while let Some(chunk) = stream.next().await {
            let chunk = chunk.map_err(|e| format!("Stream read error: {}", e))?;
            let text = String::from_utf8_lossy(&chunk);

            for line in text.lines() {
                let data = match line.strip_prefix("data: ") {
                    Some(d) => d,
                    _ => continue,
                };
                if data == "[DONE]" {
                    continue;
                }
                let parsed: StreamChunk = match serde_json::from_str(data) {
                    Ok(c) => c,
                    Err(_) => continue,
                };
                if let Some(choice) = parsed.choices.into_iter().next() {
                    if let Some(c) = choice.delta.content {
                        print!("{}", c);
                        use std::io::Write;
                        let _ = std::io::stdout().flush();
                        content.push_str(&c);
                    }
                    for tc in choice.delta.tool_calls.unwrap_or_default() {
                        merge_stream_tool_call(&mut tool_call_buf, tc);
                    }
                }
            }
        }

        if !content.is_empty() {
            println!();
        }

        let (tool_calls, raw_tool_calls) = finalize_stream_tool_calls(&tool_call_buf);

        let assistant_message = if raw_tool_calls.is_empty() {
            json!({ "role": "assistant", "content": content })
        } else {
            json!({
                "role": "assistant",
                "content": if content.is_empty() { Value::Null } else { Value::String(content.clone()) },
                "tool_calls": raw_tool_calls,
            })
        };

        Ok(AssistantResponse {
            assistant_message,
            content,
            tool_calls,
        })
    }

    #[allow(dead_code)]
    async fn post_json(&self, endpoint: &str, payload: &Value) -> Result<String, String> {
        let mut req = self
            .http
            .post(endpoint)
            .header(CONTENT_TYPE, "application/json")
            .json(payload);

        if let Some(env_name) = &self.profile.api_key_env {
            if let Ok(key) = std::env::var(env_name) {
                if !key.trim().is_empty() {
                    req = req.header(AUTHORIZATION, format!("Bearer {}", key));
                }
            }
        }

        let resp = req
            .send()
            .await
            .map_err(|e| format!("LLM request failed: {}", e))?;
        let status = resp.status();
        let body = resp
            .text()
            .await
            .map_err(|e| format!("Failed to read LLM response: {}", e))?;

        if !status.is_success() {
            return Err(format!("LLM returned {}: {}", status, body));
        }
        Ok(body)
    }
}

pub fn tool_definitions() -> Vec<Value> {
    vec![
        json!({
            "type": "function",
            "function": {
                "name": "inspect",
                "description": "Inspect system hardware and OS state.",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "target": {
                            "type": "string",
                            "enum": ["system", "cpu", "memory", "gpu", "disk", "network"]
                        }
                    }
                }
            }
        }),
        json!({
            "type": "function",
            "function": {
                "name": "propose",
                "description": "Create a proposed Nix config change without applying it.",
                "parameters": {
                    "type": "object",
                    "required": ["description"],
                    "properties": {
                        "description": { "type": "string" }
                    }
                }
            }
        }),
        json!({
            "type": "function",
            "function": {
                "name": "apply",
                "description": "Apply a proposal by ID. Requires user confirmation.",
                "parameters": {
                    "type": "object",
                    "required": ["proposal_id"],
                    "properties": {
                        "proposal_id": { "type": "string" },
                        "no_rebuild": { "type": "boolean" }
                    }
                }
            }
        }),
        json!({
            "type": "function",
            "function": {
                "name": "audit",
                "description": "Read audit log entries.",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "action": { "type": "string", "enum": ["list", "show"] },
                        "id": { "type": "string" },
                        "limit": { "type": "integer" }
                    }
                }
            }
        }),
        json!({
            "type": "function",
            "function": {
                "name": "memory",
                "description": "Manage persistent memory files. Use 'consolidate' when usage exceeds 80% to deduplicate entries.",
                "parameters": {
                    "type": "object",
                    "required": ["action"],
                    "properties": {
                        "action": { "type": "string", "enum": ["show", "add", "replace", "remove", "consolidate"] },
                        "file": { "type": "string", "enum": ["memory", "user"] },
                        "section": { "type": "string" },
                        "content": { "type": "string" },
                        "target": { "type": "string" },
                        "replacement": { "type": "string" }
                    }
                }
            }
        }),
        json!({
            "type": "function",
            "function": {
                "name": "rollback",
                "description": "Roll back the system to the previous NixOS generation via nixos-rebuild switch --rollback. Requires confirmation."
            }
        }),
    ]
}

pub fn build_system_prompt(config_dir: impl AsRef<Path>, inspect_summary: &str) -> String {
    let memory = CoreMemory::load(config_dir).unwrap_or_else(|_| CoreMemory {
        memory: String::new(),
        user: String::new(),
        memory_path: "".into(),
        user_path: "".into(),
    });

    format!(
        "You are AgntOS, an OS-aware assistant for NixOS.\n\
Rules:\n\
- Use tools for all system actions.\n\
- Always propose before applying system changes.\n\
- Applying changes or rolling back requires user confirmation.\n\
- Keep responses concise and clear.\n\
- Update memory when learning stable facts (hardware, preferences, known issues).\n\
\n\
System snapshot:\n{}\n\
\n\
MEMORY.md ({}% used):\n{}\n\
\n\
USER.md ({}% used):\n{}\n\
\n\
Available tools: inspect, propose, apply, audit, memory, rollback\n",
        inspect_summary.trim(),
        memory.usage_percent(MemoryFile::Memory),
        if memory.memory.trim().is_empty() {
            "(empty)"
        } else {
            memory.memory.trim()
        },
        memory.usage_percent(MemoryFile::User),
        if memory.user.trim().is_empty() {
            "(empty)"
        } else {
            memory.user.trim()
        }
    )
}

pub fn normalize_endpoint(base: &str) -> String {
    let trimmed = base.trim_end_matches('/');
    if trimmed.ends_with("/chat/completions") {
        trimmed.to_string()
    } else {
        format!("{}/chat/completions", trimmed)
    }
}

// ── Request helpers ─────────────────────────────────────────────────────────

fn completion_payload(
    profile: &ModelProfile,
    messages: &[Value],
    tools: &[Value],
    stream: bool,
) -> Value {
    json!({
        "model": profile.model,
        "messages": messages,
        "tools": tools,
        "tool_choice": "auto",
        "stream": stream,
        "max_tokens": profile.max_tokens,
        "temperature": profile.temperature,
    })
}

#[allow(dead_code)]
fn parse_completion_response(body: &str) -> Result<AssistantResponse, String> {
    let parsed: ChatResponse = serde_json::from_str(body)
        .map_err(|e| format!("Failed to parse LLM response JSON: {}", e))?;

    let choice = parsed
        .choices
        .into_iter()
        .next()
        .ok_or_else(|| "LLM response had no choices".to_string())?;

    if choice.message.role != "assistant" {
        return Err(format!(
            "Unexpected message role from model: {}",
            choice.message.role
        ));
    }

    let content = choice.message.content.unwrap_or_default();
    let mut tool_calls = Vec::new();
    let mut raw_tool_calls = Vec::new();

    if let Some(raw_calls) = choice.message.tool_calls {
        for raw in raw_calls {
            let args = serde_json::from_str::<Value>(&raw.function.arguments)
                .unwrap_or_else(|_| json!({}));
            tool_calls.push(ToolCall {
                id: raw.id.clone(),
                name: raw.function.name.clone(),
                arguments: args,
            });
            raw_tool_calls.push(json!({
                "id": raw.id,
                "type": "function",
                "function": {
                    "name": raw.function.name,
                    "arguments": raw.function.arguments,
                }
            }));
        }
    }

    let assistant_message = if raw_tool_calls.is_empty() {
        json!({ "role": "assistant", "content": content })
    } else {
        json!({
            "role": "assistant",
            "content": if content.is_empty() { Value::Null } else { Value::String(content.clone()) },
            "tool_calls": raw_tool_calls,
        })
    };

    Ok(AssistantResponse {
        assistant_message,
        content,
        tool_calls,
    })
}

// ── Streaming SSE types ─────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
struct StreamChunk {
    choices: Vec<StreamChoice>,
}

#[derive(Debug, Deserialize)]
struct StreamChoice {
    delta: StreamDelta,
}

#[derive(Debug, Deserialize, Default)]
struct StreamDelta {
    #[serde(default)]
    content: Option<String>,
    #[serde(default)]
    tool_calls: Option<Vec<StreamToolCall>>,
}

#[derive(Debug, Deserialize)]
struct StreamToolCall {
    index: Option<usize>,
    #[serde(default)]
    id: Option<String>,
    #[serde(default)]
    function: Option<StreamFunctionDelta>,
}

#[derive(Debug, Deserialize)]
struct StreamFunctionDelta {
    #[serde(default)]
    name: Option<String>,
    #[serde(default)]
    arguments: Option<String>,
}

fn merge_stream_tool_call(buf: &mut Vec<StreamToolCall>, incoming: StreamToolCall) {
    let idx = incoming.index.unwrap_or(0);
    while buf.len() <= idx {
        buf.push(StreamToolCall {
            index: Some(buf.len()),
            id: None,
            function: None,
        });
    }
    let slot = &mut buf[idx];
    if incoming.id.is_some() {
        slot.id = incoming.id;
    }
    if let Some(inc_func) = incoming.function {
        let f = slot.function.get_or_insert_with(|| StreamFunctionDelta {
            name: None,
            arguments: None,
        });
        if inc_func.name.is_some() {
            f.name = inc_func.name;
        }
        if let Some(args) = inc_func.arguments {
            let cur = f.arguments.get_or_insert_with(String::new);
            cur.push_str(&args);
        }
    }
}

fn finalize_stream_tool_calls(buf: &[StreamToolCall]) -> (Vec<ToolCall>, Vec<Value>) {
    let mut tool_calls = Vec::new();
    let mut raw = Vec::new();
    for tc in buf {
        let id = tc.id.clone().unwrap_or_default();
        let name = tc
            .function
            .as_ref()
            .and_then(|f| f.name.clone())
            .unwrap_or_default();
        let args_str = tc
            .function
            .as_ref()
            .and_then(|f| f.arguments.clone())
            .unwrap_or_default();
        let args = serde_json::from_str::<Value>(&args_str).unwrap_or_else(|_| json!({}));
        tool_calls.push(ToolCall {
            id: id.clone(),
            name: name.clone(),
            arguments: args,
        });
        raw.push(json!({
            "id": id,
            "type": "function",
            "function": { "name": name, "arguments": args_str },
        }));
    }
    (tool_calls, raw)
}
