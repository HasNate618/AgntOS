# AgntOS State

## Current Phase

Project foundation (Phase 0): spec-driven project structure and initial Nix/Rust scaffold.

## Decisions

- Use NixOS as base distribution.
- Package as full distro with custom scripts, programs, configs, and image outputs.
- Start with KDE Plasma only (v0).
- Use Rust for core tools and daemons.
- Prefer Kirigami for initial GUI, allow CLI/TUI for dev speed.
- Prioritize Hermes-like agent with custom OS integration before AI Anywhere or automation.
- Let `agntctl` directly edit Nix config with guardrails.
- Support cloud endpoints/API keys and local model management.
- Include model routing layer for task-specific model assignment.
- Brand as cutting-edge new OS for users who want to get into AI.
- Keep specs flexible — the project is speculative.
- Use git version control from the start.
- Dev VM root is Nix-built; local source mounted for iteration, not replacing the OS image.

## Resolved Questions

- **Agent interface**: Staged — CLI/TUI first for dev speed, then Kirigami GUI on top once the backend is solid.
- **Config model**: Dedicated AgntOS tree — `agntctl` writes to its own config directory (e.g. `/etc/agntos/`), imported by the user's main config. Does not edit `configuration.nix` directly in v0.
- **Hermes integration**: Prototype with Hermes behind the scenes to validate the product, then decide whether to fork, wrap, or replace.

## Still Open

- Should `agntctl` apply Nix changes immediately after approval or stage changes for a separate apply step?
- What is the minimum local model backend for v0: Ollama, llama.cpp, vLLM, or another?
- Should `agntd` run as a user service, system service, or split service?
- What is the first approval mechanism: Polkit, sudo-like prompts, Plasma auth dialogs, or a custom daemon?
- What should the first audit log format be?
- How should AgntOS represent skills: markdown spec directories, typed Rust plugins, executable tools, or hybrid?
- What is the first Plasma theme/brand direction?
- When should ISO work start relative to VM foundation?

## Assumptions

- NixOS can express the AgntOS distro shape without requiring a separate build system.
- A dev VM with shared source folders is better than a mutable VM root.
- Typed OS control tools keep agent behavior safer and easier to debug than raw shell.
- Plasma integration is sufficient for v0.
- Direct Nix config editing is acceptable for prototypes; safety mechanisms added as design hardens.

## Risks

- Nix learning curve may slow early development.
- Direct config editing can corrupt user config without careful staging, formatting, and backups.
- Rust daemons + Kirigami UI may require cross-language integration decisions.
- Model management can expand too fast and distract from the OS foundation.
- Safety and rollback flows may be underestimated.
- Full distro packaging may pull attention from the agent foundation if started too early.

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

## Preferences

- Keep specs concise but decision-rich.
- Use flexible language where product shape is not yet proven.
- Favor small milestones that prove real OS behavior.

## Research Notes

- Hermes Agent: useful for skills, memory, model switching, cron, toolsets, self-improvement patterns.
- NixOS: fits agent-managed system state because it is declarative and rollback-oriented.
- BlueBuild/Fedora Atomic: useful fallback reference but not the chosen foundation.
- XDG portals, KWin scripting, Hyprland IPC: relevant for later AI Anywhere and automation phases.