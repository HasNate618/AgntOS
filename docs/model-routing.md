# AgntOS Model Routing

AgntOS uses a user-managed TOML file at `/etc/agntos/models.toml`.

Providers hold OpenAI-compatible **endpoint URLs** and an optional **API key environment variable**. Concrete **model ids** are chosen in the Control Centre chat dropdown (fetched live from each provider's `/v1/models` API).

Example:

```toml
[default]
endpoint = "http://127.0.0.1:8081/v1"
model = ""
api_key_env = "AGNTOS_API_KEY"

[profiles.gateway]
endpoint = "http://10.0.0.45/bifrost/v1"
api_key_env = "AGNTOS_API_KEY"
model = ""

[routing]
chat = "gateway"
inspect = "gateway"
```

Legacy flat tables (`[gateway]` beside `[routing]`) still parse; new writes use `[profiles.<name>]`.

## Fields

- `endpoint`: OpenAI-compatible base URL (e.g. `.../v1`)
- `model`: selected model id (often empty until chosen in the UI)
- `api_key_env`: optional environment variable containing an API key

## Commands

- `agntctl model list`
- `agntctl model add <name> --endpoint <url> [--api-key-env VAR]`
- `agntctl model route <task>`

Use `--config-dir <path>` to inspect a non-default config tree.
