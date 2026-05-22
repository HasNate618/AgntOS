# Control Centre GUI v2 — Design

## Architecture

```
┌─────────────────────────────────────────────────────────┐
│ React UI (shadcn + assistant-ui Thread)                 │
│  ThreadList │ ChatChrome + ModelSelector │ Thread       │
│  ModelsPage │ Status │ Proposals │ Activity             │
└────────────┬────────────────────────────────────────────┘
             │ Tauri invoke + events
┌────────────▼────────────────────────────────────────────┐
│ agntos-cc (Rust): commands.rs + pi_bridge.rs            │
│  Pi RPC: prompt, sessions, set_model, get_available_*   │
│  agntctl: proposals, audit, model list/add/remove       │
└────────────┬────────────────────────────────────────────┘
             │
     ┌───────┴────────┐
     ▼                ▼
  Pi (RPC)      /etc/agntos (group agntos)
  ~/.pi/agent   proposals, memory, models.toml
```

## Permissions

- Nix `users.groups.agntos`; `proposals/` and `memory/` dirs `0775 root:agntos`
- `developer` in `agntos` group on dev-vm
- Pi extension reads `AGNTOS_CONFIG_DIR` (set by pi_bridge)

## Streaming mapping

| Pi event | assistant-ui part |
|----------|-------------------|
| `thinking_delta` | `reasoning` |
| `text_delta` | `text` |
| tool events | `tool-call` |

## Models

- Source of truth: `/etc/agntos/models.toml` (`ModelsConfig`)
- On Pi start: sync profiles → `~/.pi/agent/models.json` for Pi provider list
- Chat selector calls `set_model` with `provider/modelId` from Pi response

## Sessions

- Pi stores JSONL under `~/.pi/agent/sessions/`
- `list_sessions` Tauri command scans directory (no Pi RPC required)
- Switch clears frontend runtime messages; Pi loads session history on next prompt
