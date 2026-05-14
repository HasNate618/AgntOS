# AgntOS State

## Current Phase

Phase 1 (Agent OS Foundation): implementing LLM-powered agent with Hermes-style memory, model routing config, and OS tool interface.

## Completed (Phase 0)

- Nix flake with `agntos-dev-vm` and `agntos-plasma` configurations.
- Modules: base, desktop-plasma, vm, branding.
- Profiles: dev-vm, plasma.
- Rust workspace: agnt-common, agntctl, agntd.
- `agntctl inspect`: hardware/OS inspection (CPU, memory, GPU, disks, network, JSON output).
- `agntctl propose`: keyword/template proposal generation (install/remove/enable/disable + generic).
- `agntctl apply`: read proposal JSON, write Nix files, run nixos-rebuild, log to audit.
- `agntctl audit`: JSONL audit log, list/show.
- `agntd` CLI agent loop: INTENT → `agntctl` subprocess, confirmation flow.
- QEMU VM: boots, SSH on port 2222, shared folder mounts, tools build and run inside VM.
- 19 tests, 0 warnings.
- End-to-end verified (propose → apply → audit) inside VM.

## Decisions

### Model & Agent Architecture

- **Memory model**: Hermes-style bounded curated files (`MEMORY.md` + `USER.md`), frozen snapshot at session start. Not a vector database. Rationale: the agent needs *system facts* (GPU model, installed packages, past config changes) — these are small, structured, and always relevant. A bounded file forces curation. Retrieval is unnecessary overhead for this use case.
- **LLM interface**: OpenAI-compatible `/v1/chat/completions` API. Chosen because it's the widest-supported standard — works with OpenAI, Anthropic via proxy, Ollama, vLLM, LiteLLM, and all local inference servers. Users configure their own endpoint.
- **No bundled models or defaults**: `localhost:8081` is a user's personal endpoint example, not a built-in default. The user must configure at least one model profile in `models.toml` for the agent to work.
- **Agent loop**: LLM-powered tool calling, not keyword matching. Tools: inspect, propose, apply, audit, memory. Each tool maps to an `agntctl` subprocess call.
- **Confirmation**: required for `apply` and destructive operations. `inspect`, `propose`, `audit`, `memory` tools proceed without confirmation.

### Config & Storage

- **Config format**: TOML for model routing (`/etc/agntos/models.toml`). JSON for proposals. Markdown for memory files.
- **Model routing**: task-class based TOML config. Task classes map to model profiles. Profiles define endpoint, model name, API key env var, max_tokens, temperature.
- **Audit log**: JSONL at `/var/log/agntos/audit.jsonl`. Append-only. Read by `agntctl audit`.
- **Session search**: SQLite FTS5 (`/etc/agntos/memory/sessions.db`). On-demand, not in-prompt. No embedding model required.
- **Config ownership**: dedicated AgntOS tree (`/etc/agntos/`). `agntctl` never touches the user's `configuration.nix`.

### Architecture

- **OS changes go through Nix**: every mutation is a declarative Nix config edit. Rollback is `nixos-rebuild --rollback`.
- **Tool calls, not shell commands**: `agntctl` is a typed interface. The agent cannot run arbitrary bash.
- **Rust for the control plane**: system daemons and tools in Rust. Memory-safe, no runtime dependencies, single binary deployment.
- **Propose-then-apply**: changes are staged as proposals first, applied after review. The agent creates a proposal, shows it to the user, then applies on confirmation.

### Skills

- **Deferred**: skills system comes after core agent is stable. Phase 1 delivers the foundation — Phase 2 or later adds procedural memory.
- **First approach**: markdown spec directories (inspired by Hermes SKILL.md) in `/etc/agntos/skills/`.

## Resolved Questions

| Question | Resolution |
|---|---|
| Agent interface order | CLI/TUI first → Kirigami later. Now: LLM-powered REPL replaces basic keyword matching. |
| Config scope | `/etc/agntos/` owned by agntctl. No edits to `configuration.nix`. |
| Hermes integration | Inspect architecture patterns only. No code reuse from Hermes Agent — different domain (OS control vs chat agent). |
| Memory format | Bounded curated files (MEMORY.md + USER.md), not vector DB. |
| Model routing format | TOML, not JSON or Nix. |
| LLM API standard | OpenAI-compatible `/v1/chat/completions`. |
| Confirmation model | Interactive CLI prompt for `apply`. |
| Session search | SQLite FTS5, not vector search. |
| Agent binary path | `find_agntctl()` prefers `target/release` over `target/debug`. |

## Still Open

- **Local model backend**: Ollama, llama.cpp, vLLM, or adapter pattern (Phase 2).
- **Skills format**: markdown spec directories, typed Rust plugins, or hybrid.
- **Approval UI**: CLI prompt sufficient for Phase 1. Plasma auth dialog or Polkit for Phase 3.
- **Service model**: `agntd` as user service vs system service. Current: runs as user process. May need split daemon for privileged operations.
- **ISO timing**: after Phase 1 or after Phase 3.
- **Kirigami UI timing**: after Phase 1 or parallel.

## Risks

- **LLM latency**: synchronous API calls make the agent feel slow if the endpoint is far or under load. Mitigation: streaming, local model option.
- **Memory corruption**: agent writes bad facts to memory, then acts on them. Mitigation: security scanning, capacity pressure forces consolidation, user can inspect/edit memory directly.
- **Prompt injection**: user input tricks the LLM into making dangerous tool calls. Mitigation: confirmation for destructive ops, security scanning on memory writes, typed tool interface limits blast radius.
- **Nix rebuild time**: even simple config changes require `nixos-rebuild test` which can be slow. Mitigation: `--no-rebuild` flag for write-only changes, user chooses when to rebuild.
- **Model routing complexity**: too many task classes or profiles could confuse users. Mitigation: sensible defaults for the v0 classes, power users can customize.
- **Agent loop infinite regress**: LLM keeps calling tools without producing output. Mitigation: tool call depth limit, timeout.
- **Nix learning curve**: we need Nix expertise for the config layer while being Rust-focused. Mitigation: modular separation, Nix layer is thin.

## Assumptions

- NixOS can express the AgntOS distro shape without requiring a separate build system.
- A dev VM with shared source folders is better than a mutable VM root.
- Typed OS control tools keep agent behavior safer and easier to debug than raw shell.
- Plasma integration is sufficient for v0.
- Direct Nix config editing is acceptable for prototypes; safety mechanisms added as design hardens.
- Bounded curated memory (< 4KB total) is sufficient for OS agent context. The system facts the agent needs are small and stable.
- OpenAI-compatible API will remain the widest-supported LLM interface standard.

## Deferred Ideas

- AI Anywhere screen selection (Phase 5).
- Full desktop automation (Phase 6).
- Hyprland Lab edition.
- Developer edition.
- Agent-created widgets.
- Self-improving skills.
- Skills marketplace.
- Remote/cloud agent execution.
- Background automation in nested compositor or VM.
- Skills system (Phase 2+).

## Preferences

- Keep specs concise but decision-rich.
- Use flexible language where product shape is not yet proven.
- Favor small milestones that prove real OS behavior.
- No abstraction before it's needed.
