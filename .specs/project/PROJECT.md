# AgntOS Project

## What Is AgntOS?

AgntOS is an **AI-native operating system** built on NixOS. It's not a chatbot that runs on Linux — it's an OS where the AI agent is a first-class system component, like the kernel or the display manager.

The agent can:
- **Inspect** the system (hardware, config, services, logs)
- **Remember** system state across sessions (structured, curated memory)
- **Propose** OS changes through typed tools (`agntctl`)
- **Apply** changes by generating and rebuilding Nix config
- **Audit** every action with rollback guidance

The user **chooses their own LLM** — the agent connects to any OpenAI-compatible API endpoint the user configures. No bundled models, no vendor lock-in.

## Vision

AgntOS is for users who want a cutting-edge OS that helps them use AI and manage their system without becoming Linux administrators first.

The product centers on a Hermes-like local agent with deep OS integration. The agent understands the system, manages configuration, routes model tasks, and safely applies changes through structured tools.

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
| **Agent framework** | Hermes Agent, LangChain, AutoGPT | None are designed for OS-level control. They're for chatbots and API workflows. We need typed OS tools (with system privileges), audit logging, declarative config mutation, and a feedback loop through Nix. |
| **Agent memory** | MemGPT/Letta, Zep, Honcho, RAG on vector DBs | All optimized for conversation memory ("what did the user say last Tuesday?"). An OS agent needs *system state* memory: GPU driver version, disk layout, package list, network config. These are facts, not chat history. Using a vector DB for this is over-engineered — a small curated file is faster, cheaper, and more reliable. |
| **OS control** | Ansible, Chef, Puppet | Datacenter automation tools, not interactive desktop agents. They assume a skilled admin writing playbooks. `agntctl` is designed for an LLM to call as typed tools with structured JSON inputs/outputs. |
| **Model routing** | LiteLLM, OpenRouter | Proxy/gateway tools for developers. AgntOS needs model routing as an *OS feature* — configured in system config, visible in the desktop settings, auditable. |
| **Config management** | Manual Nix editing, home-manager | Both require the user to edit config files. The whole point of AgntOS is the agent does this *for you*, with safety guardrails and rollback built in. |

The closest analogy is `systemd` — it didn't invent init scripts, it invented a new model for service management. We're inventing the interface between an AI agent and the OS kernel.

## Product Principles

- **Agent-first, OS-aware**: the agent is a system operator with typed OS tools, not just a chat window.
- **Declarative and recoverable**: every OS change goes through Nix config. Rollback is always one command away.
- **Memory is curated, not retrieved**: the agent maintains a small, always-in-context memory file. Bounded capacity forces quality. No vector DB needed.
- **User-owned models**: the user configures their own endpoints. AgntOS is vendor-neutral. `localhost:8081` is an example user setup, not a default.
- **Safe autonomy**: changes are proposed before applied, every mutation is audited, rollback guidance is automatic.
- **Plasma first**: v0 targets KDE Plasma only.
- **Rust core**: system daemons and control tools are built in Rust — memory-safe, no runtime dependencies.
- **Nix first**: AgntOS packages as a full distro with custom scripts, programs, configs, and ISO output.

## Locked Decisions

- Base OS: NixOS.
- Desktop: KDE Plasma (v0 only).
- Language: Rust.
- GUI: Kirigami (TUI/CLI allowed for dev speed).
- Agent priority: Hermes-like agent with custom OS integration first.
- OS control: `agntctl` directly edits Nix config with guardrails.
- **Memory model**: Hermes-style bounded curated files (`MEMORY.md` + `USER.md`), frozen snapshots at session start. Not a vector database.
- **LLM interface**: OpenAI-compatible `/v1/chat/completions` API. Users configure their own endpoint. No bundled models.
- **Model routing**: task-class based TOML config (`/etc/agntos/models.toml`). Not hardcoded.
- **Agent loop**: LLM-powered tool calling (inspect/propose/apply/audit/memory tools). Not keyword matching.
- **Confirmation**: `apply` and destructive operations require user confirmation.
- **Session search**: SQLite FTS5 for "did we discuss X" queries. No embedding model.
- **Skills system**: deferred beyond Phase 1. Core agent must work first.
- Branding: cutting-edge new OS for users who want to get into AI.
- Scope: AI Anywhere and desktop automation come after the OS-integrated agent foundation.

## Open Design Space

- Exact local model backend: Ollama, llama.cpp, vLLM, or backend-agnostic adapter (Phase 2).
- Skills format: markdown spec directories, typed Rust plugins, or hybrid (deferred).
- Approval UI: Polkit, sudo-like prompts, Plasma auth dialogs, or custom daemon.
- First artifact: dev ISO, installable alpha ISO, or both.

## Success Definition

The first credible AgntOS milestone: a bootable Plasma-based NixOS VM where a user can open the agent and safely ask it to inspect hardware, reason about models, edit Nix-backed OS config through `agntctl`, apply approved changes, and understand rollback options. The agent remembers system state between sessions and can explain what it knows.
