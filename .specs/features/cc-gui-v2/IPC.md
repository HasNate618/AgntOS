# Control Centre IPC audit

| UI surface | Tauri command | Status |
|------------|---------------|--------|
| Chat send | `send_prompt` | OK |
| Chat cancel | `send_abort` | OK |
| Model dropdown | `get_available_models`, `set_model` | OK (Pi RPC response via `agent:rpc-response`) |
| Sessions list | `list_sessions` | OK (filesystem scan) |
| New session | `new_session` | OK |
| Switch session | `switch_session` | OK |
| Approval | `send_extension_ui_response` | OK |
| Status page | `get_system_info`, `get_connection_status` | OK |
| Proposals | `list_proposals`, `apply_proposal` | OK |
| Activity | `list_audit_entries`, `rollback_to` | OK |
| Models page | `get_models_config`, `add_model_provider`, `probe_provider_models`, `remove_model_profile` | OK |
| Chat model picker | `list_model_catalog`, `set_chat_model`, `set_model` | OK |

Not wired in UI: `send_steer` (reserved).
