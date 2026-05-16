# AgntOS State

## Current Phase

Phase 1 complete (foundation + agent loop + general tools + daemon mode + surgical rollback). Phase 1 expansions underway (memory/provenance, watchdogs). Phase 2 (model management) is next.

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
