# AgntOS Project

## What Is AgntOS?

AgntOS is an **AI-native operating system** built on NixOS. It's not a chatbot that runs on Linux — it's an OS where the AI agent is a first-class system component, like the kernel or the display manager.

The agent can:
- **Inspect** the system (hardware, config, services, logs)
- **Read, write, and edit** any file on the system
- **Run shell commands** for diagnostics and debugging
- **Remember** system state across sessions (structured, curated memory)
- **Propose and apply** OS changes through Nix config + nixos-rebuild
- **Roll back** any change to the previous generation
- **Audit** every mutation with full traceability

The user **chooses their own LLM** — the agent connects to any OpenAI-compatible API endpoint the user configures. No bundled models, no vendor lock-in.

## Philosophy

AgntOS is inspired by three projects that each represent a different dimension of "mutation":

| Project | What it mutates | AgntOS inherits |
|---|---|---|
| **Nix** | System state (declarative, reproducible, rollbackable) | Every OS change is a Nix generation. Rollback is one command away. |
| **Pi** (coding agent) | Itself (4 primitives, agent builds its own tools) | Small core of typed tools. The agent uses `read_file`/`write_file`/`edit_file`/`run_bash` for everything else. No plugin system — the agent extends itself. |
| **Hermes** | Memory (curated facts, bounded capacity) | `MEMORY.md` + `USER.md` as always-in-context agent knowledge. Frozen snapshot per session. Capacity pressure forces curation quality. |

AgntOS is **an OS that mutates** — system state, agent memory, and agent capabilities all evolve through the same typed tool interface.

## Vision

AgntOS is for users who want a cutting-edge OS that helps them use AI and manage their systems without becoming Linux administrators first.

The product centers on a local agent with deep OS integration. The agent understands the system, inspects files and logs, runs diagnostics, manages Nix configuration, applies changes safely, and learns across sessions.

AgntOS should feel like a new operating system, not a chatbot bolted onto Linux.

## Target User

Primary:
- Normal ambitious users who want to be more productive with AI.
- AI-curious users who want local model support, cloud endpoint flexibility, and a guided desktop experience.
- Creators, students, researchers, and technical beginners who want Linux without memorizing sysadmin details.

Secondary:
- Power users who want direct access to Nix internals, model routing, logs, and agent skills.

Out of scope for v0: enterprise fleet management, Hyprland/power-user edition, full autonomous desktop control, universal DE abstraction beyond Plasma.

## Why Build Our Own Stack?

Existing tools solve adjacent problems but none are designed for an agent-native OS. Here's what we'd be composing if we went with existing stacks — and why we don't:

| Area | Existing options | Why we build our own |
|---|---|---|
| **Agent framework** | Hermes Agent, LangChain, AutoGPT, Pi coding agent | None are designed for OS-level control. They're for coding or chat workflows. We need typed OS tools (with system privileges), Nix-integrated config mutation, audit logging, and a feedback loop through nixos-rebuild. Pi's 4-primitive philosophy is closest — but it has no OS mutation tools. |
| **Agent memory** | MemGPT/Letta, Zep, Honcho, RAG on vector DBs | All optimized for conversation memory ("what did the user say last Tuesday?"). An OS agent needs *system state* memory: GPU driver version, disk layout, package list, network config. These are facts, not chat history. Using a vector DB for this is over-engineered — a small curated file is faster, cheaper, and more reliable. |
| **OS control** | Ansible, Chef, Puppet | Datacenter automation tools, not interactive desktop agents. They assume a skilled admin writing playbooks. `agntctl` is designed for an LLM to call as typed tools with structured JSON inputs/outputs. |
| **Model routing** | LiteLLM, OpenRouter | Proxy/gateway tools for developers. AgntOS needs model routing as an *OS feature* — configured in system config, visible in the desktop settings, auditable. |
| **Config management** | Manual Nix editing, home-manager | Both require the user to edit config files. The whole point of AgntOS is the agent does this *for you*, with safety guardrails and rollback built in. |

The closest analogy is `systemd` — it didn't invent init scripts, it invented a new model for service management. We're inventing the interface between an AI agent and the OS kernel.

## Product Principles

- **Agent-first, OS-aware**: the agent is a system operator with typed OS tools, not just a chat window.
- **Small core, maximum reach**: 10 tools total (6 OS-native + 4 general primitives). The agent uses `run_bash` for anything without a dedicated tool. No plugin system — the agent extends itself.
- **Declarative and recoverable**: every OS change goes through Nix config. Rollback is always one command away.
- **Memory is curated, not retrieved**: the agent maintains a small, always-in-context memory file. Bounded capacity forces quality. No vector DB needed.
- **User-owned models**: the user configures their own endpoints. AgntOS is vendor-neutral. `localhost:8081` is an example user setup, not a default.
- **Autonomous by default**: only `apply` and `rollback` ask for confirmation. Everything else executes immediately. The system prompt instructs the agent to chain propose+apply without pausing.
- **Native integration**: designed for Plasma integration (system tray, KRunner, notifications, Kirigami window). Terminal REPL is the development interface, not the final UX.
- **Plasma first**: v0 targets KDE Plasma only.
- **Rust core**: system daemons and control tools are built in Rust — memory-safe, no runtime dependencies.
- **Nix first**: AgntOS packages as a full distro with custom scripts, programs, configs, and ISO output.
- **Everything audited**: every mutation (file write, bash execution, Nix change) is logged to JSONL. Audit log and memory serve different purposes and are NOT the same system.

## Tool Catalog

| Category | Tool | Purpose | Confirmation? |
|---|---|---|---|
| OS | `inspect` | Read system state (CPU, RAM, GPU, disks, network) | No |
| OS | `propose` | Stage a Nix config change | No |
| OS | `apply` | Apply staged change + nixos-rebuild | **Yes** |
| OS | `rollback` | nixos-rebuild --rollback | **Yes** |
| OS | `audit` | Read the mutation log | No |
| OS | `memory` | Manage persistent agent knowledge | No |
| General | `read_file` | Read a file's contents | No |
| General | `write_file` | Create or overwrite a file | No |
| General | `edit_file` | Replace a string in a file | No |
| General | `run_bash` | Execute a shell command | No |

**10 tools total.** 6 OS-native, 4 general primitives (directly from Pi's philosophy: read, write, edit, bash). The agent uses `run_bash` for `ls`, `grep`, `find`, `systemctl`, `journalctl`, `dmesg`, and any command without a dedicated tool.

## Locked Decisions

- Base OS: NixOS.
- Desktop: KDE Plasma (v0 only).
- Language: Rust.
- GUI: Kirigami (terminal REPL allowed for dev speed).
- Agent priority: OS-integrated agent with Hermes memory + Pi-inspired tool philosophy.
- OS control: `agntctl` directly edits Nix config with guardrails.
- **Memory model**: Hermes-style bounded curated files (`MEMORY.md` + `USER.md`), frozen snapshots at session start. Not a vector database. Not the same thing as the audit log.
- **Audit log**: Separate from memory. Immutable record of every mutation. JSONL at `/var/log/agntos/audit.jsonl`.
- **LLM interface**: OpenAI-compatible `/v1/chat/completions` API. Users configure their own endpoint. No bundled models.
- **Model routing**: task-class based TOML config (`/etc/agntos/models.toml`). Not hardcoded.
- **Agent loop**: LLM-powered tool calling. 10 tools, one agent, no modes.
- **Confirmation**: only `apply` and `rollback` require user confirmation. `write_file`, `edit_file`, `run_bash` execute immediately.
- **Session search**: SQLite FTS5 for "did we discuss X" queries. No embedding model.
- **Skills system**: deferred. First approach: markdown directories (Hermes SKILL.md pattern).
- **Pi philosophy**: 4 general primitives replace dozens of specialized tools. Agent builds its own capabilities by writing skills and scripts. No MCP, no plugin marketplace.
- Branding: an OS that mutates. "Mutate your OS."
- Scope: AI Anywhere and desktop automation come after the OS-integrated agent foundation.

## Open Design Space

- Exact local model backend: Ollama, llama.cpp, vLLM, or backend-agnostic adapter (Phase 2).
- Skills format: markdown spec directories, typed Rust plugins, or hybrid (deferred).
- Approval UI: CLI prompt now, Plasma auth dialog or Polkit later.
- First artifact: dev ISO, installable alpha ISO, or both.
- Plasma integration surface: system tray icon, KRunner plugin, Kirigami chat window, notification bridge.
- Socket mode for agntd: Unix socket or D-Bus for GUI frontends.

## Success Definition

The first credible AgntOS milestone: a bootable Plasma-based NixOS VM where a user can open the agent and ask it to inspect hardware, read logs, debug a service, install a package through Nix, and roll back if needed. The agent remembers system state between sessions. Every mutation is auditable.
