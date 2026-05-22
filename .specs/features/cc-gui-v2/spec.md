# Control Centre GUI v2

## Problem

The React migration is functional but has regressions and missing Phase 3.2 surfaces: agent text renders inside the reasoning block, `agntos_propose` fails with EACCES, branding uses a placeholder icon, chat has no session list or model picker, and model routing is CLI-only.

## Goals

- Fix streaming part mapping (text vs reasoning) and proposal write permissions
- Use official AgntOS logo in chrome and app icon
- Session sidebar (list, new, switch) wired to Pi RPC
- Model selector in chat composer (assistant-ui pattern)
- Models/providers settings page (endpoint URL, API key env, profiles, routing)
- IPC audit: every UI action maps to an implemented Tauri command

## Requirements

### CC-GUI-001: Proposal permissions
WHEN the agent calls `agntos_propose` as the desktop user THEN proposals SHALL be written under `AGNTOS_CONFIG_DIR/proposals` without EACCES.

### CC-GUI-002: Streaming parts
WHEN `text_delta` events arrive THEN content SHALL append to a `text` part; only `thinking_*` events SHALL use `reasoning` parts.

### CC-GUI-003: Branding
WHEN the app renders THEN the AgntOS SVG logo SHALL appear in sidebar and chat chrome; bundle icons SHALL use brand assets.

### CC-GUI-004: Sessions
WHEN the user opens chat THEN a collapsible session list SHALL show Pi sessions with New Session and switch actions via `new_session` / `switch_session` / `list_sessions`.

### CC-GUI-005: Model selector
WHEN the user composes a message THEN a model dropdown SHALL list Pi providers/models from `get_available_models` and apply selection via `set_model`.

### CC-GUI-006: Models page
WHEN the user opens Models THEN they SHALL add/remove OpenAI-compatible profiles (endpoint, model id, API key env) persisted to `models.toml` via agntctl.

### CC-GUI-007: IPC audit
Each page SHALL only invoke commands registered in `main.rs`; missing commands SHALL be added or UI disabled.

## Out of scope

- Kirigami / agntos-settings changes
- agntd replacement (Pi remains backend)
- Dark/light theme toggle
