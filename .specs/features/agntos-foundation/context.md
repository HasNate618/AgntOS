# AgntOS Foundation Context

## User Decisions Captured

- Package AgntOS as a full distribution with custom scripts, programs, and configs.
- Use NixOS for the distro base.
- Use git version control from the start.
- Prioritize a Hermes-like agent with custom OS integration before AI Anywhere or full automation.
- Support both cloud model endpoints/API keys and local model management.
- Include model routing so users can assign models to different task types.
- Brand AgntOS as a cutting-edge AI OS for users who want to get into AI.
- Use Rust as the primary implementation language.
- Use Kirigami as the likely GUI toolkit.
- Allow future TUI or alternate UI exploration, but keep it out of v0 scope.
- Let `agntctl` directly edit Nix config.
- Keep user config representation undecided.
- Target Plasma only for v0.

## Clarified Direction

AgntOS should not begin as a generic assistant app. The operating system layer is central: agent behavior should be built around managing a NixOS system safely.

The first build target should be a dev VM rather than immediately perfecting the ISO. The ISO remains a product requirement, but the VM gives faster iteration and avoids slow bare-metal testing during foundation work.

## Open Questions For Later Discussion

- Should the first agent interface be a CLI, a minimal Kirigami chat app, or both?
- Should user-facing config be Nix, TOML/YAML that generates Nix, or a hybrid?
- What should the AgntOS-managed Nix config path be?
- Should `agntctl` use Polkit for privileged operations in v0?
- Should `agntd` run as a user service, system service, or split services?
- Should secrets use KDE Wallet, age/sops-nix, pass, or another store?
- Should the first local model backend be Ollama for ease or llama.cpp for tighter packaging?

## Flexibility Rules

- Capture uncertainty instead of resolving it prematurely.
- Prefer interfaces that allow implementation swaps later.
- Keep first milestones small enough that agents can develop and verify them.
- Avoid decisions that force support for multiple desktops before Plasma works.
