<p align="center">
  <picture>
    <source media="(prefers-color-scheme: dark)" srcset="docs/AgntOS-Banner.svg">
    <img src="docs/AgntOS-Banner.svg" width="70%" alt="AgntOS -- AI-native operating system">
  </picture>
</p>

<p align="center">
  <strong>AgntOS is a NixOS distribution built for AI-driven system management. An LLM agent proposes configuration changes, you approve them, and NixOS applies them declaratively with a full audit trail and instant rollback.</strong>
</p>

- [The Problem](#the-problem)
- [What AgntOS Does](#what-agntos-does)
- [How It Works](#how-it-works)
- [Why NixOS?](#why-nixos)
- [Quick Start](#quick-start)
- [Architecture](#architecture)
- [Tool Catalog](#tool-catalog)
- [Memory & Provenance](#memory--provenance)
- [License](#license)

---

## The Problem

Configuring and maintaining a Linux system is hard. There are dozens of config files spread across the filesystem, imperative commands that leave the system in an unknown state, and no built-in way to undo mistakes.

Existing AI tools help you write config files and run commands, but they operate on top of whatever OS you already have. They can suggest changes, but they can't guarantee the system is in a known state before or after. They have no persistent memory — every conversation starts fresh. And they have no undo — one wrong command is one wrong command.

AgntOS solves this by building the OS around the agent, not bolting the agent onto the OS.

## What AgntOS Does

AgntOS is a NixOS distribution where an LLM agent has read and write access to the operating system configuration — safely, declaratively, and reversibly.

**System management.** The agent proposes configuration changes as Nix files, validates them before apply, and NixOS applies them deterministically. No imperative commands, no unknown state.

**Persistent memory.** The agent maintains curated markdown files for system conventions and user preferences. These are loaded into every LLM call, so the agent remembers what you like between sessions.

**Self-extension.** The agent can write new tools and capabilities using the same tools it uses for everything else — `write_file` and `run_bash`. No plugin SDK.

Every change is audited with your original request attached. You can always ask "why is this installed?" and get a correct answer.

## How It Works

AgntOS works because NixOS provides a clean boundary between the agent and the system:

**The agent owns one directory: `/etc/agntos/`.** It writes Nix files there. NixOS imports them. That's the full contract. The agent never touches arbitrary system files.

**Every change goes through a pipeline:**
1. Propose — agent generates intent, Nix syntax is validated
2. Confirm — user reviews before any files change
3. Apply — Nix files written, old files snapshotted, `nixos-rebuild` runs
4. Audit — action recorded with the user's prompt attached
5. Undo — surgical revert or whole-system `nixos-rebuild --rollback`

**The agent's tools match the problem.** Six of the ten tools (`inspect`, `propose`, `apply`, `rollback`, `audit`, `memory`) exist because NixOS config management requires structured data that bash can't represent. The other four (`read_file`, `write_file`, `edit_file`, `run_bash`) cover everything else.

## Why NixOS?

NixOS is the only OS where an agent can treat the entire machine as a declarative artifact. Every other OS relies on imperative package managers and scattered config files — the agent cannot safely own any part of them.

| Property | How NixOS provides it |
|---|---|
| **Clean contract** | The agent writes `.nix` files into `/etc/agntos/`. NixOS imports them. No side effects, no guessing where a change landed. |
| **Generations = free rollbacks** | Every `nixos-rebuild` creates a bootable checkpoint. The audit log provides file-level undo alongside whole-system revert. |
| **Deterministic builds** | Same flake lock, same system. Agent changes are reproducible and verifiable before apply. |
| **Everything is an option** | Configuring a firewall, changing the hostname, enabling a service — all Nix options. One tool covers 95% of system config. |

NixOS is the foundation. The product is what the agent can do on top of it.

## Quick Start

### Build and run the dev VM

```bash
git clone https://github.com/your-org/agntos
cd agntos
nix build --impure .#nixosConfigurations.agntos-dev-vm.config.system.build.vm
./result/bin/run-agntos-dev-vm
```

### Login

- **User**: `developer` / password: `agntos`
- **SSH**: `ssh -p 2222 developer@localhost`

### Configure the LLM

```bash
sudo cp /etc/agntos/models.toml.example /etc/agntos/models.toml
sudo editor /etc/agntos/models.toml
export AGNTOS_API_KEY=your-key
```

### First interactions

```bash
agntctl inspect system
agntctl propose "install htop"

export AGNTOS_API_KEY=your-key
agntd
```

## Architecture

```mermaid
flowchart LR
    User["User"] --> agntd
    agntd -->|tool calls| agntctl["agntctl"]
    agntctl -->|reads/writes| etc["/etc/agntos/"]
    etc -->|nixos-rebuild| NixOS["NixOS"]
```

### Components

| Component | Role |
|---|---|
| **agntd** | LLM agent daemon. Accepts prompts via REPL or Unix socket. Assembles system prompt from memory + system snapshot + tool definitions. Dispatches tool calls. |
| **agntctl** | OS control CLI. 10 tools. All agent tool calls run `agntctl` as a subprocess. |
| **agnt-common** | Shared types (audit entries, config proposals, memory) serialized as JSON across the subprocess boundary. |

### The /etc/agntos/ directory

```
/etc/agntos/
  packages/          Per-package Nix modules
  services/          Per-service enable/disable modules
  options/           Arbitrary NixOS option overrides
  proposals/         Staged config proposals (JSON)
  memory/            MEMORY.md + USER.md + sessions.db (SQLite FTS5)
  audit.jsonl        Append-only action log
  models.toml        LLM endpoint configuration
  flake-info         Flake URI for nixos-rebuild --flake
```

## Tool Catalog

| Tool | Confirmation | Purpose |
|---|---|---|
| `inspect` | No | Read system state (CPU, memory, GPU, disk, network) |
| `propose` | No | Stage a Nix config change with syntax validation |
| `apply` | Yes | Apply staged proposal, write Nix files, rebuild |
| `rollback` | Yes | NixOS generation rollback or surgical undo |
| `audit` | No | Query action history, search by prompt/path/package |
| `memory` | No | Curate MEMORY.md and USER.md |
| `read_file` | No | Read file contents |
| `write_file` | No | Create or overwrite a file |
| `edit_file` | No | Replace text in a file |
| `run_bash` | No | Execute arbitrary shell commands |

Only `apply` and `rollback` require user confirmation.

## Memory & Provenance

### Core Memory

Two markdown files loaded into every LLM call:

| File | Capacity | Purpose |
|---|---|---|
| `MEMORY.md` | 2,200 chars | System facts, conventions, constraints |
| `USER.md` | 1,375 chars | User preferences, workflow patterns, intent |

The agent curates these files directly. When memory exceeds 80% capacity, the agent consolidates: deduplicates and merges similar entries.

### What belongs in memory

Preferences and intent, not inspectable system state:

- "User prefers Helix over Neovim" — stored
- "This is a Rust development machine" — stored
- "GPU is QEMU Bochs" — not stored (re-inspectable via `inspect`)
- "8GB RAM" — not stored (re-inspectable)
- "htop is installed" — not stored (re-inspectable)

Storing inspectable state duplicates the real data and goes stale. The agent can always read the current state from the system.

### Provenance

Every `apply` stores the user's original prompt in the audit entry. The agent retrieves it when asked "why was this done?":

```bash
agntctl audit search --query "btop"
```

## License

Copyright (C) 2026  AgntOS contributors

This program is free software: you can redistribute it and/or modify
it under the terms of the GNU General Public License as published by
the Free Software Foundation, either version 3 of the License, or
(at your option) any later version.
