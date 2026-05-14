# AgntOS State

## Current Phase

Phase 1 complete. Moving into Phase 1 expansion (general tools) and Phase 2 preparation.

## Completed (Phase 0)

- Nix flake with `agntos-dev-vm` and `agntos-plasma` configurations.
- Modules: base, desktop-plasma, vm, branding, agent.
- Profiles: dev-vm, plasma.
- Rust workspace: agnt-common, agntctl, agntd.
- All Phase 0 and Phase 1 tasks implemented and VM-validated.

## Completed (Phase 1 — Agent OS Foundation)

- `agntctl inspect`: hardware/OS inspection (CPU, memory, GPU, disks, network, JSON output).
- `agntctl propose`: keyword/template proposal generation (install/remove/enable/disable + generic).
- `agntctl apply`: read proposal JSON, write Nix files, flake-aware nixos-rebuild, log to audit.
- `agntctl audit`: JSONL audit log, list/show/limit.
- `agntctl model`: list configured profiles, route task to profile.
- `agntctl memory`: show/add/replace/remove/consolidate agent memory files.
- `agntctl rollback`: list generations, roll back to previous generation (flake-aware).
- `agntd` LLM-powered agent: streaming, tool-calling loop, confirmation gates, conversation persistence.
- Hermes-style memory: bounded curated files, frozen snapshots, security scanning, consolidation.
- SQLite FTS5 session store: turn persistence, `history <query>` search, prior context injection.
- Flake-aware rebuild: `/etc/agntos/flake-info` → `nixos-rebuild --flake <uri> --impure`.
- Auto-start: systemd user service in dev VM.
- QEMU VM: boots, SSH on port 2222, shared folder mounts, all tools build and run.
- 34 tests, zero warnings.

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
4. Store stable system facts in memory

### Audit vs Memory (separate systems)

| | Audit log | Memory |
|---|---|---|
| **What** | Immutable record of WHAT happened | Curated knowledge of WHAT IS |
| **Purpose** | Accountability, rollback, debugging | Continuity across sessions |
| **Writer** | Every mutation automatically | Agent via `memory` tool |
| **Capacity** | Unlimited (append-only JSONL) | Bounded (2200 + 1375 chars) |
| **In context** | No — on-demand via `audit` tool | Yes — frozen snapshot per session |

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

## Still Open

- Local model backend: Ollama, llama.cpp, vLLM, or adapter pattern.
- Skills format: markdown spec directories, typed Rust plugins, or hybrid.
- Plasma integration surface: system tray, KRunner, Kirigami window, notifications.
- ISO timing and distro packaging.
- Socket mode for agntd: Unix socket or D-Bus.

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
