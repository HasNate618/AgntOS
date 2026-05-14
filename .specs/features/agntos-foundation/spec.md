# Feature Spec: AgntOS Foundation

## Scope

Build the initial foundation for AgntOS as a NixOS-based AI-native distribution. This feature covers the repository structure, NixOS flake shape, Plasma-only dev VM, Rust control tooling direction, and first OS-integrated agent architecture.

This feature intentionally leaves room for discovery. It should establish stable seams for future work without prematurely deciding every UX, model backend, or agent implementation detail.

## Goals

- Create a NixOS-based distro foundation that can produce a development VM and later an ISO.
- Define the first OS control surface, `agntctl`, as a Rust tool that directly edits AgntOS-managed Nix config.
- Define the first agent architecture around a Hermes-like local assistant with structured OS tools.
- Define how model customization and routing will fit into the system.
- Keep Plasma as the only v0 desktop target.
- Keep future AI Anywhere and automation work out of the critical path.

## Non-Goals

- Building a complete installable ISO in the first foundation task.
- Implementing AI Anywhere screen selection.
- Implementing full desktop automation.
- Supporting multiple desktop environments.
- Implementing a production-grade local model manager.
- Forking or embedding Hermes Agent before evaluating the AgntOS-specific control layer.

## Requirements

### AGF-001: NixOS Distribution Source

AgntOS must be represented as a Nix flake that can own system modules, packages, VM profiles, and later ISO outputs.

Acceptance criteria:
- The repository contains a `flake.nix`.
- The flake has a dev VM output or a clearly documented placeholder for it.
- The flake structure separates modules, packages, and profiles.

### AGF-002: Plasma-Only v0 Profile

AgntOS v0 must target KDE Plasma only.

Acceptance criteria:
- The first system profile enables Plasma.
- No GNOME, Hyprland, Xfce, or multi-DE abstraction is required for v0.
- Future desktop expansion is documented as deferred.

### AGF-003: Fast Dev VM Workflow

AgntOS must support VM-based development without replacing the VM root with a local directory.

Acceptance criteria:
- The VM root is built from Nix.
- The local source repo can be mounted into the VM for fast iteration.
- Release-mode packaging remains separate from dev-mode source mounting.

### AGF-004: Rust OS Control Tool

AgntOS must define `agntctl` as the Rust command-line control tool used by agents and users for OS operations.

Acceptance criteria:
- `agntctl` is defined as a package target.
- `agntctl` is responsible for direct Nix config edits in controlled AgntOS-managed locations.
- `agntctl` has room for inspect, propose, apply, audit, and rollback-related commands.

### AGF-005: Direct Nix Editing With Guardrails

`agntctl` must directly edit Nix config, but the system must be designed around reviewability and recovery.

Acceptance criteria:
- Direct edits are scoped to an AgntOS-managed config area unless explicitly expanded later.
- Planned changes can be previewed before application.
- Applied changes are recorded in an audit log.
- Rollback guidance is available after changes.

### AGF-006: Hermes-Like Agent First

The first product milestone must prioritize the agent with OS integration.

Acceptance criteria:
- The agent architecture includes skills, memory, model routing, and tool execution concepts.
- The first agent tools interact with `agntctl`.
- AI Anywhere and desktop automation remain later milestones.

### AGF-007: Customizable Model Routing

AgntOS must support flexible cloud/local model configuration and routing.

Acceptance criteria:
- Model routing is represented as task-class assignments.
- Cloud models can be configured through endpoints and API keys.
- Local model support is included in the architecture even if the first backend is not final.
- Users can override defaults.

### AGF-008: Spec-Driven Flexibility

The foundation must preserve flexibility because the project is speculative.

Acceptance criteria:
- Open questions are captured in project state.
- Specs avoid overcommitting to a single UI implementation beyond Plasma/Kirigami direction.
- Future choices are documented as extension points rather than hidden assumptions.

### AGF-009: Git From The Start

The project must use git version control from the beginning.

Acceptance criteria:
- The repository is initialized with git.
- Generated build outputs are ignored.
- No commit is required unless explicitly requested.

## First Demo Story

As an AgntOS developer, I can boot a Plasma-based AgntOS VM, open an early assistant or CLI, and ask it to inspect the OS and propose a safe Nix-backed change through `agntctl`.

## Risks

- Nix complexity may slow early development.
- Direct config editing can become dangerous without strong boundaries.
- A Kirigami-first UI could slow the first agent milestone.
- Local model management can become a large project by itself.
- Reusing Hermes too early could couple AgntOS to another project before the OS-control model is stable.
