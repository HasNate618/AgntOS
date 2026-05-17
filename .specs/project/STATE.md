# AgntOS State

## Current Phase

**Settings Stabilization (SS track)** — Aligning the agntos-settings GUI with its own Phase 3 spec. Phase 3 v1 is code-complete but the runtime path drifted from the spec: ad hoc `serde_json::Value` state, missing turn lifecycle, broken proposal rendering, refresh crashes. The stabilization track replaces the dual model/bridge split with a single AppSession source of truth, explicit turn state, and proper event routing.

Phase 3.2 (model routing page, memory viewer, cxx-qt bridge) is deferred until stabilization is complete.

### Stabilization Track Status

- SST-01 through SST-04 (foundation): Not started
- SST-05 through SST-08 (QML migration): Not started
- SST-09 through SST-10 (event routing, reconnect): Not started
- SST-11 through SST-12 (testing, validation): Not started

See `.specs/features/settings-stabilization/` for spec, design, and tasks.

## Completed (Phase 0)

- Nix flake with `agntos-dev-vm` and `agntos-plasma` configurations.
- Modules: base, desktop-plasma, vm, branding, agent.
- Profiles: dev-vm, plasma.
- Rust workspace: agnt-common, agntctl, agntd.
- All Phase 0 and Phase 1 tasks implemented and VM-validated.

## Completed (Phase 1 — Agent OS Foundation)

- `agntctl inspect`: hardware/OS inspection (CPU, memory, GPU, disks, network, JSON output).
- `agntctl propose`: keyword/template proposal generation (install/remove/enable/disable/set + generic).
- `agntctl apply`: read proposal JSON, write Nix files, flake-aware nixos-rebuild, log to audit, track files_written/files_deleted.
- `agntctl audit`: JSONL audit log, list/show/limit.
- `agntctl model`: list configured profiles, route task to profile.
- `agntctl memory`: show/add/replace/remove/consolidate agent memory files.
- `agntctl rollback`: list generations, roll back to previous generation (flake-aware), surgical undo via audit log.
- `agntd` LLM-powered agent: streaming, tool-calling loop, confirmation gates, conversation persistence.
- Core Memory: bounded curated files, frozen snapshots, security scanning, consolidation.
- SQLite FTS5 session store: turn persistence, `history <query>` search, prior context injection.
- Flake-aware rebuild: `/etc/agntos/flake-info` → `nixos-rebuild --flake <uri> --impure`.
- Auto-start: systemd user service in dev VM.
- QEMU VM: boots, SSH on port 2222, shared folder mounts, all tools build and run.
- 55 tests, zero warnings.

## Completed (Phase 1 Expansions)

### Pi-Inspired General Tools
- 4 minimal primitives: `read_file`, `write_file`, `edit_file`, `run_bash`.
- Exposed as LLM tools alongside Nix-native tools.
- Audit logging for write/bash operations.
- 41 tests for sys module.

### Daemon Mode & Error Handling
- `agntd --socket <path>`: Unix domain socket listener for one-shot JSON requests.
- Systemd user service uses `--socket` mode.
- Rollback friendly errors: "No NixOS generations found" on transient VMs.
- APPLY_CANCELLED fix: agent told not to retry when apply is cancelled.
- Eval runbook: 14/14 checks passing.

### Surgical Rollback, Nix Validation, Option Templates
- `agntctl rollback undo`: audit-log surgical revert (reverse files_written, warn on files_deleted).
- `--persist` flag: `nixos-rebuild switch` instead of `test` for persistent changes.
- `validate_nix()`: generated Nix files validated with `nix-instantiate --parse` before acceptance.
- `propose set <option> <value>`: arbitrary NixOS option templates (string, bool, int, raw).
- `/etc/agntos/options/` directory imported in base.nix.
- `files_to_delete`, `files_written`, `files_deleted` tracked in AuditEntry/ConfigProposal.
- 55 tests, zero warnings.
- Eval: 14/14 checks passing.

### Memory & Provenance
- `prompt` and `rationale` fields added to AuditEntry with `#[serde(default)]`.
- Audit search returns prompt content for provenance.
- System prompt updated: "do NOT store inspectable facts in memory".
- End-of-session auto-consolidation: on REPL exit, LLM reviews session turns and extracts facts.

### Proactive Self-Healing (Watchdogs)
- Background watchdog thread with tokio runtime, independent of REPL/socket.
- Three checks: `systemctl --failed`, `df -h / > 95%`, `dmesg | grep -i oom`.
- Triaged via LLM: CONFIG_ERROR → `agntctl propose`; TRANSIENT → logged.
- Notifications via eprintln and watchdog.log.

### Home Manager Integration
- `propose set-home-option <option> <value>` keyword added to propose.rs.
- Files go to `/etc/agntos/home/`, imported by base.nix.
- User configurable via `AGNTOS_USER` env var (defaults to "primary").

## Completed (Phase 2 — Model Management CLI)

- `agntctl model add` / `remove` / `set-route` / `suggest` commands.
- Profile CRUD with TOML serialization (blocks removing "default").
- Task-class routing assignment.
- Hardware-aware model recommendations via inspect.
- API keys remain env-var based (`api_key_env` field).
- 65 tests, zero warnings.

## Completed (Phase 3 v1 — Kirigami Settings)

### Overview

Phase 3 v1 delivers the first Kirigami GUI for AgntOS: a chat-driven control center with dashboard pages for status, proposals, and audit history. The architecture uses a bidirectional NDJSON protocol over persistent Unix domain socket connections to agntd.

### Components

**agntos-settings crate** (`crates/agntos-settings/`):
- QML UI files using Kirigami 2.20: main window with global drawer, ChatPage, StatusPage, ProposalsPage, ActivityPage.
- Rust backend: Unix socket connection with exponential backoff reconnect, NDJSON protocol codec, session state machine.
- Data models: ChatModel (streaming tokens, tool call lifecycle, approval gating), ProposalModel (reads /etc/agntos/proposals/), StatusModel (agent/system/watchdog state), AuditModel (parses audit.jsonl entries).
- 24 unit tests + 6 integration tests = 30 total.
- Nix package and NixOS module (`agntos.settings.enable`).

**agntd protocol extension**:
- Persistent session mode: NDJSON loop alongside legacy one-shot protocol.
- Approval gate: `Arc<Mutex<Option<ApprovalGate>>>` shared between reader thread and chat processing thread.
- Chat turns execute in a separate thread to avoid blocking the reader loop.
- Spin-wait with 5-minute timeout prevents resource leaks on disconnect.
- 27 tests (22 existing + 5 new).

**Shared wire protocol** (`agnt_common::wire`):
- 7 ClientMessage types (init, chat, approve, dismiss, status, audit, cancel).
- 10 ServerMessage types (session_ready, status_response, token, tool_call, tool_result, approval_request, turn_complete, audit_response, event, error).
- 6 roundtrip/edge-case tests.

### Key Decisions

- **NDJSON over Unix socket** — stays true to "bash is king" philosophy, backward compatible with socat/scripts.
- **Separate thread for chat processing** — avoids deadlock between approval gate wait and reader loop.
- **Plain Rust structs for data models** — QML binding (cxx-qt/qmetaobject) deferred to Phase 3.2; models are testable and complete.
- **Approval gate with timeout** — 5-minute max wait, client disconnect handled gracefully.
- **QML pages as standalone files** — loaded by Qt runtime; no build-time code generation.

### Wire Protocol

Client → Server: `{"type":"init"}`, `{"type":"chat","prompt":"..."}`, `{"type":"approve","proposal_id":"p-..."}`, `{"type":"status","target":"system"}`, `{"type":"audit","action":"list",...}`, `{"type":"cancel"}`

Server → Client: streaming `token` messages, `tool_call` with status transitions (Running→Done), `tool_result`, `approval_request`, `turn_complete`, `event`, `audit_response`, `status_response`, `session_ready`, `error`

Backward compat: existing one-shot `{"prompt":"..."}` → `{"response":"..."}` still works without changes.

### Test Results

- agnt_common: 12 tests (6 existing + 6 wire protocol)
- agntd: 27 tests (22 existing + 5 approval gate/proposals)
- agntos-settings: 30 tests (24 unit + 6 integration)
- Total: 69 tests passing

### What's Deferred to Phase 3.2

- Model routing configuration page (`agntctl model` CLI suffices)
- Memory viewer/editor (`agntctl memory` CLI suffices)
- cxx-qt bridge for direct Rust↔QML bindings (QML files are ready, Rust models are ready, bridge code is TBD)
- Push event wiring from watchdog to GUI (EventSender scaffolded in agent.rs, not yet producing events)
- Multiple simultaneous GUI connections (v1 supports one per socket path)

## Decisions

### Architecture Philosophy

Inspired by three projects:
- **Nix**: declarative, reproducible, rollbackable system mutations.
- **Pi (coding agent by Mario Zechner)**: minimal core (4 primitives), agent builds its own tools via code generation. No MCP, no plugin marketplace.
- **Hermes**: bounded curated memory, frozen snapshots, agent-curated knowledge.

AgntOS is **an OS that mutates** — system state, agent memory, and agent capabilities all evolve through typed tools.

### Tool Catalog (10 tools)

| Category | Tool | Confirmation? |
|---|---|---|
| OS-native (6) | `inspect`, `propose`, `apply`, `rollback`, `audit`, `memory` | Only `apply` and `rollback` |
| General (4) | `read_file`, `write_file`, `edit_file`, `run_bash` | None |

Pi-inspired: 4 general primitives replace dozens of specialized tools. The agent uses `run_bash` for `ls`, `grep`, `find`, `systemctl`, `journalctl`, `dmesg`, etc. No `list_dir` or `search_files` — bash handles both.

### Confirmation Model

- `apply` and `rollback`: interactive confirmation prompt. LLM does not gate — the system handles it.
- Everything else: executes immediately. Audit log provides accountability.

### System Prompt Philosophy

The system prompt instructs the agent to:
1. Chain propose+apply without pausing ("do not ask permission, just execute")
2. Prefer structured tools over raw bash for file operations
3. Use `run_bash` for any command without a dedicated tool
4. Store user preferences and non-inspectable context in memory (not inspectable system facts)

### Audit vs Memory (separate but complementary)

| | Audit log | Memory |
|---|---|---|
| **What** | Immutable record of WHAT happened and WHY | Curated knowledge of current state |
| **Purpose** | Accountability, rollback, debugging, provenance | Continuity across sessions |
| **Writer** | Every mutation automatically (+ prompt/rationale) | Agent via `memory` tool |
| **Capacity** | Unlimited (append-only JSONL) | Bounded (2200 + 1375 chars) |
| **In context** | No — on-demand via `audit` tool | Yes — frozen snapshot per session |
| **Provenance** | prompt + rationale fields track the "why" | N/A — preferences and intent only |

### Memory Architecture (Decision: Single System, No Hermes)

- **One memory system, not two.** The agent curates `MEMORY.md` and `USER.md` via the `memory` tool. No separate Hermes-style background extraction pipeline — the agent is the best judge of what matters, in-context.
- **Don't store inspectable facts.** Memory is for preferences, intent, and context that can't be derived from system state. `agntctl inspect` gives fresh system info on every session — no need to store it in memory.
- **End-of-session auto-consolidation.** When the session ends (socket close or idle), the agent reviews the conversation and updates memory automatically. This replaces the need for a background extraction system.
- **Provenance at the source.** `prompt` and `rationale` fields on `AuditEntry` capture the "why" when the action happens, rather than trying to infer it later.

### Agent Loop

LLM-powered tool calling, not keyword matching. 10 tools, one agent, no modes/profiles/user-facing complexity.

### Plasma Integration (architected, not yet implemented)

Current: terminal REPL. Future:
- System tray icon (agent status)
- KRunner plugin (Alt+Space → agent commands)
- Kirigami chat window (full GUI)
- Notification bridge (D-Bus)
- agntd `--socket` mode for GUI frontends

### Branding

"Mutate your OS." Three layers:
1. System mutation: propose → apply → nixos-rebuild
2. Memory mutation: agent learns → `memory add` → next session knows more
3. Self mutation: agent writes skills → gains new capabilities

## Resolved Questions

| Question | Resolution |
|---|---|
| Agent interface order | CLI/TUI first → Kirigami later. Now: LLM-powered REPL. |
| Config scope | `/etc/agntos/` owned by agntctl. No edits to `configuration.nix`. |
| Memory format | Bounded curated files (MEMORY.md + USER.md), not vector DB. |
| Model routing format | TOML, not JSON or Nix. |
| LLM API standard | OpenAI-compatible `/v1/chat/completions`. |
| Confirmation model | Only `apply` and `rollback` gate. System prompt instructs agent to chain ops without asking. |
| Session search | SQLite FTS5, not vector search. |
| General tools | 4 primitives (read_file, write_file, edit_file, run_bash) — Pi-inspired. No list_dir or search_files. |
| Audit vs memory | Separate systems. Audit = immutable mutation log. Memory = curated agent knowledge. |
| Agent modes/profiles | None. One agent, 10 tools, flat. No user-facing complexity. |
| Skills system | Deferred. Markdown spec directories (Hermes SKILL.md). Agent builds its own. |
| Memory architecture | Single system, agent-curated. No Hermes-style background extraction. |
| Introspection approach | Bash is king. Agent uses `run_bash` for `ps`, `systemctl`, etc. No structured JSON APIs. |

## Still Open

- Secure API key storage (file-based key files, keyring integration).
- Local model backend adapter tuning per hardware config.
- Skills format: markdown spec directories, typed Rust plugins, or hybrid.
- Plasma integration surface: system tray, KRunner, Kirigami window, notifications.
- ISO timing and distro packaging.

## Risks

- **LLM latency**: synchronous API calls. Mitigation: streaming, local model option.
- **Prompt injection in bash**: LLM could craft destructive commands. Mitigation: system prompt forbids destructive patterns, audit log provides accountability, rollback provides recovery.
- **Nix rebuild time**: `nixos-rebuild` can be slow. Mitigation: `--no-rebuild` flag, user chooses when to rebuild.
- **Agent loop infinite regress**: tool call depth limit (5).

## Deferred Ideas

- AI Anywhere screen selection (Phase 5).
- Full desktop automation (Phase 6).
- Hyprland Lab edition.
- Developer edition.
- Agent-created widgets.
- Self-improving skills.
- Skills marketplace.
- MCP integration (agent can build its own via bash if needed).

## Preferences

- Keep specs concise but decision-rich.
- Favor small milestones that prove real OS behavior.
- No abstraction before it's needed.
- Small core, maximum reach.
