# AgntOS Model Routing

AgntOS uses a user-managed TOML file at `/etc/agntos/models.toml`.

AgntOS does not ship with a default endpoint. Each user provides their own endpoint(s), model IDs, and optional API key environment variable names.

Example:

```toml
[default]
endpoint = "http://localhost:8081/v1"
model = "qwen2.5-coder:14b"
api_key_env = "AGNTOS_API_KEY"
max_tokens = 4096
temperature = 0.7

[fast]
endpoint = "http://localhost:8082/v1"
model = "qwen2.5:7b"
max_tokens = 2048
temperature = 0.3

[routing]
inspect = "fast"
propose = "default"
apply = "default"
chat = "default"
memory = "default"
```

## Fields

- `endpoint`: OpenAI-compatible base URL (for example `.../v1`)
- `model`: model identifier sent to the provider
- `api_key_env`: optional environment variable containing an API key
- `max_tokens`: completion token cap
- `temperature`: sampling temperature

## Commands

- `agntctl model list`
- `agntctl model route <task>`

Use `--config-dir <path>` to inspect a non-default config tree.
