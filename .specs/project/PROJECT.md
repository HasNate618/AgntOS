# AgntOS Project

## Vision

AgntOS is a NixOS-based AI-native Linux distribution for users who want a cutting-edge OS that helps them enter and use AI without becoming Linux administrators first.

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

## Product Principles

- Agent-first, OS-aware: the agent is a system operator with typed OS tools, not just a chat window.
- Declarative and recoverable: OS changes go through Nix config and support rollback.
- Flexible by design: early specs define stable boundaries while leaving implementation details open.
- User-owned models: users bring cloud endpoints/API keys and manage local models.
- Safe autonomy: preview, approval, audit logs, and rollback are core surfaces.
- Plasma first: v0 targets KDE Plasma only.
- Rust core: system daemons and control tools are built in Rust.
- Nix first: AgntOS packages as a full distro with custom scripts, programs, configs, and ISO output.

## Locked Decisions

- Base OS: NixOS
- Desktop: KDE Plasma (v0 only)
- Language: Rust
- GUI: Kirigami (TUI/CLI fallback allowed for dev speed)
- Agent priority: Hermes-like agent with custom OS integration first
- OS control: `agntctl` directly edits Nix config with guardrails
- Model strategy: fully customizable routing across local and cloud
- Branding: cutting-edge new OS for users who want to get into AI
- Scope control: AI Anywhere and desktop automation come after the OS-integrated agent foundation

## Open Design Space

- User-facing config format: plain Nix, higher-level generated format, or both
- Hermes reuse level: backend, fork, or inspiration only
- First artifact: dev ISO, installable alpha ISO, or both
- First local model backend: Ollama, llama.cpp, vLLM, or backend-agnostic adapter
- `agntctl` autonomy level without human review

## Success Definition

The first credible AgntOS milestone: a bootable Plasma-based NixOS VM where a user can open the agent and safely ask it to inspect hardware, reason about models, edit Nix-backed OS config through `agntctl`, apply approved changes, and understand rollback options.