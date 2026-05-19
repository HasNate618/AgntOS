# Feature Spec: Pi + Tauri Migration

## Scope

Replace the current AgntOS agent backend (agntd) and GUI (agntos-settings) with **Pi as a hidden agent backend** and **Tauri as the frontend**. Pi is an internal dependency — the user and LLM must never know Pi exists. All visible identity, tools, system prompt, and UI are purely AgntOS.

This is a **full replacement** of the agntd + agntos-settings stack. The NixOS modules, agntctl, and agnt-common crates remain mostly intact.

## Goals

- **Replace agntd** with Pi in RPC mode as the hidden agent engine — streaming, session trees, compaction, tool calling, auto-retry all come for free.
- **Pi is invisible.** The LLM sees only AgntOS tools (agntos_*), the AgntOS system prompt. No Pi brand, no Pi default tools, no Pi AGENTS.md. Zero Pi leakage.
- **Replace agntos-settings** (QML/Kirigami) with a Tauri v2 app (Rust backend + Svelte frontend) — eliminates all Qt/Wayland/QML flakiness.
- **Preserve AgntOS identity** — NixOS mutations, `propose → apply → rollback` workflow, audit trail, system inspector, agent memory.
- **Preserve agntctl** as the NixOS control layer — the Pi extension calls `agntctl` under the hood for ALL operations.
- **Batteries-included Nix packaging** — Pi (Node.js), Tauri app, and agntctl all packaged via Nix. Pi is a private dependency, not exposed to the user's PATH.
- **Better developer experience** — LLMs can write Svelte/TSX far more reliably than QML; Pi extensions are just TypeScript.

## Non-Goals

- Rewriting agntctl in TypeScript (it stays in Rust).
- Supporting non-Plasma desktops in v1 (Plasma stays the only target).
- Building an ISO installer (still deferred).
- Multitenancy or remote access (agent runs as the desktop user, same as now).
- Local model management beyond what Pi already provides via `models.json`.
- Making Pi a user-facing tool (it's a hidden dependency).

## Decisions (from Design Review)

### D-1: Pi is a hidden backend only
Pi is launched with flags that strip its identity:
- `--no-builtin-tools`: Disables Pi's read/write/edit/bash/grep/find/ls. Only `agntos_*` tools exist.
- `--no-context-files`: Prevents Pi from loading any AGENTS.md. No cross-contamination with user's Pi config.
- `--system-prompt <string>`: Completely replaces Pi's default system prompt with pure AgntOS instructions.
- `--extension <path>`: Loads the agntos-tools extension which registers ALL tools the LLM can use.

The agntos-tools extension covers everything the agent needs:
- `agntos_bash`, `agntos_read`, `agntos_write` replace Pi's built-in bash/read/write
- `agntos_propose`, `agntos_apply`, `agntos_rollback` for NixOS mutations
- `agntos_inspect`, `agntos_audit`, `agntos_memory` for system operations

The user can also install Pi independently for their own coding work. They are separate concerns.

### D-2: Session persistence — Pi JSONL for chat, agntctl audit for system ops
- Pi handles conversation sessions natively (JSONL files in `~/.pi/agent/sessions/`).
- `agntctl audit` continues writing to `/var/log/agntos/audit.jsonl` — only system mutations (propose, apply, rollback), not chat history.
- New chat = new Pi session (`new_session` RPC). Resume = `switch_session`.
- One-off prompts without persistence = `--no-session` flag.

### D-3: agntctl subprocess for all AgntOS tools
Every `agntos_*` tool in the Pi extension calls the corresponding `agntctl` command as a subprocess. agntctl already has `bash`, `read`, `write`, `edit` subcommands (see `crates/agntctl/src/sys.rs`), so Pi's built-in tools are fully replaceable.

### D-4: Keep AgntOS memory system (MEMORY.md/USER.md)
`agntctl memory` handles curated AgntOS memory. Pi's compaction handles conversation context. They're complementary.

## Requirements

### PTM-001: Pi RPC Bridge (Rust) — Pi as Hidden Backend
The Tauri app must manage Pi as a subprocess using `pi --mode rpc`, communicating via JSON over stdin/stdout. Pi must be launched with identity-stripping flags.

Acceptance criteria:
- Tauri backend spawns `pi --mode rpc --no-builtin-tools --no-context-files --system-prompt <agntos_prompt> --extension <agntos_tools>`.
- Backend sends RPC commands (prompt, steer, abort, set_model, new_session, switch_session) to Pi's stdin.
- Backend receives and parses RPC events (message_update, tool_execution_start/end, agent_end, extension_ui_request, etc.) from Pi's stdout.
- Backend exposes parsed events to the Svelte frontend via Tauri's event system.
- Backend handles Pi process lifecycle (restart on crash, graceful shutdown).
- Backend routes extension_ui_request/response for approval flow.
- All Pi errors are caught and reported as AgntOS errors, not "Pi crashed."

### PTM-002: AgntOS Pi Extension (agntos-tools)
A Pi TypeScript extension must register all AgntOS tools as Pi-native tools. The LLM only sees these tools — Pi's built-in tools are disabled.

Acceptance criteria:
- Extension registers tools: `agntos_propose`, `agntos_apply`, `agntos_rollback`, `agntos_inspect`, `agntos_audit`, `agntos_memory`, `agntos_bash`, `agntos_read`, `agntos_write`, `agntos_edit`.
- Each tool calls the corresponding `agntctl` subcommand and returns the result.
- `agntos_apply` uses `ctx.ui.confirm()` for user approval before executing.
- Tool descriptions include NixOS-specific guidance so the LLM knows when and how to use each tool (propose before apply, never mutate without proposal, etc.).
- Extension is loadable via Pi's `--extension` flag.
- pi.registerTool() + Type for parameter schema (Typebox).

### PTM-003: AgntOS System Prompt
The AgntOS system prompt must be a standalone Markdown file loaded via Pi's `--system-prompt` flag. It must:
- Never mention Pi, Claude, or any other platform.
- Only reference AgntOS tools (agntos_propose, agntos_inspect, etc.).
- Describe the NixOS mutation workflow (propose → approve → apply → rollback).
- Describe the AgntOS memory system (MEMORY.md/USER.md via agntos_memory).
- Describe available tools with usage guidelines.

### PTM-004: Svelte Chat Frontend
The Tauri webview must display a functional chat interface with proper markdown rendering, tool call visualization, and approval flow.

Acceptance criteria:
- Chat messages render with full markdown (bold, italic, code blocks, links) via `marked` + `highlight.js`.
- Streaming responses update in real-time (no polling).
- Tool calls render with name, AgntOS-branded icon, spinner while running, expandable result.
- Approval requests render as actionable cards with Approve/Reject buttons.
- Connection status indicator shows agent state, never "Pi" by name.
- Message input with send button, disabled while agent is streaming.
- All UI uses AgntOS branding — no Pi references anywhere.

### PTM-005: Proposals, Status & Activity Pages
The frontend must include pages for reviewing proposals, viewing system status, and browsing audit history.

Acceptance criteria:
- Proposals page shows pending proposals with description, nix changes summary, and Approve/Dismiss actions.
- Status page shows agent state (idle/thinking/running tool), model info, session info.
- Activity page shows audit log from `agntctl audit` with search/filter.

### PTM-006: NixOS Module Updates
The AgntOS Nix module must be updated to package and configure the new stack.

Acceptance criteria:
- Pi is packaged as a hidden dependency of AgntOS — not exposed to user's PATH unless user explicitly enables `programs.pi`.
- New `programs.agntos-cc` module for the Tauri Control Centre.
- `programs.agntctl` module remains as-is.
- Dev VM profile includes Node.js (for Pi), the Tauri app, and agntctl.
- Desktop launcher starts the Tauri app, not agntos-settings.
- Pi's config directory (`~/.pi/`) is managed by the module, separate from any user-installed Pi.

### PTM-007: agntctl Remains Stable
agntctl must continue to work as a standalone CLI tool.

Acceptance criteria:
- All existing subcommands work identically.
- agntctl can be used independently of Pi or the Tauri app.
- agntctl is still packaged via Nix and available on the system PATH.

### PTM-008: Model Configuration
The agent must use the user's configured model endpoint (currently in `/etc/agntos/models.toml`).

Acceptance criteria:
- Tauri app reads `/etc/agntos/models.toml` and passes to Pi via `--provider` and `--model` flags or `set_model` RPC.
- User can switch models from the frontend.
- Local model support (Ollama at 10.0.2.2:8081) works out of the box.

### PTM-009: Approval Gates for Destructive Operations
The agent must ask for user confirmation before applying NixOS changes.

Acceptance criteria:
- `agntos_apply` tool uses Pi's `ctx.ui.confirm()` for approval.
- Frontend renders approval cards with Approve/Reject buttons.
- User can configure auto-apply rules — deferred to v2.

### PTM-010: Graceful Degradation
If Pi is not installed or crashes, the Tauri app must handle it gracefully.

Acceptance criteria:
- Frontend shows "Agent not available" state when backend is down (never "Pi crashed").
- Auto-restart policy (up to 3 retries with backoff).
- User can manually restart from the UI.
- agntctl remains fully functional regardless of backend state.

### PTM-011: Migration Path
There must be a clean migration from the old stack to the new one.

Acceptance criteria:
- Both stacks can coexist during transition (old agntd/agntos-settings and new Pi/Tauri).
- Nix module has a feature flag or separate services for old vs new.
- Dev VM can be configured to use either stack.
- Documentation explains the migration steps.
- Session history migration: tool to convert SQLite sessions to Pi JSONL.
