# Development

## Inside the VM

When the shared folder is mounted at `/mnt/agntos-src`, the following aliases are available:

```bash
agntos          -> cd /mnt/agntos-src
agnt-build      -> cargo build --release
agnt-check      -> cargo check
agnt-test       -> cargo test
agnt-inspect    -> cargo run --bin agntctl -- inspect
agnt-agent      -> cargo run --bin agntd
```

## VM Management

### Build

```bash
export PRJ_ROOT=$(pwd)
nix build --impure .#nixosConfigurations.agntos-dev-vm.config.system.build.vm
```

`--impure` is required because `PRJ_ROOT` uses `builtins.getEnv` for the shared folder path.

### Run

```bash
./result/bin/run-agntos-dev-vm
```

First boot opens QEMU with a NixOS disk image booting into Plasma 6.

### SSH

```bash
ssh -p 2222 developer@localhost
```

Host port 2222 forwards to guest port 22.

### Shared folder

When `PRJ_ROOT` is set at build time, the repo is shared into the VM at `/mnt/agntos-src` via QEMU 9p virtio. Edit on host, build inside VM.

### Rebuild

```bash
# On host: rebuild the VM closure
nix build --impure .#nixosConfigurations.agntos-dev-vm.config.system.build.vm
# Stop and restart the VM

# Inside running VM: after modifying the flake
sudo nixos-rebuild switch --flake /mnt/agntos-src
```

### Stop

```bash
# In the VM window: Ctrl+Alt+G then close
# From host:
pkill qemu-system-x86_64
```

## Outside the VM

```bash
cargo check
cargo build
cargo test

# Build the full NixOS system (requires Nix):
export PRJ_ROOT=$(pwd)
nix build --impure .#nixosConfigurations.agntos-dev-vm.config.system.build.vm
```

## Build and run

```bash
# Build Rust tools (fast, no Nix needed on host):
cargo build --release

# Run the agent:
export AGNTOS_API_KEY=your-key
./target/release/agntd

# Run a specific tool:
./target/release/agntctl inspect system
./target/release/agntctl propose "install htop"
```

## Testing

```bash
# All tests:
cargo test

# Specific crate:
cargo test -p agntctl
cargo test -p agnt-common

# Specific test:
cargo test -- test_provenance_roundtrip
```

### Eval runbook

The eval runbook exercises all 11 tools and socket mode against a real LLM endpoint:

```bash
ssh -p 2222 developer@localhost
cd /mnt/agntos-src
sudo AGNTOS_CONFIG_DIR=/etc/agntos \
  bash .specs/features/agntos-foundation/eval-runbook.sh
```

14 checks: general tools (read/write/edit/bash), OS-native tools (inspect, propose, audit, memory), socket/daemon mode, and rollback friendly errors.

## Project structure

```
agntos/
├── flake.nix               # Nix entry point
├── Cargo.toml              # Rust workspace root
├── modules/agntos/         # NixOS modules
│   ├── base.nix            # AgntOS base (enable, configDir, edition)
│   ├── desktop-plasma.nix  # Plasma 6 Wayland desktop
│   ├── agent.nix           # agntd systemd user service
│   └── vm.nix              # QEMU VM settings + shared folder
├── profiles/               # System profiles
│   ├── dev-vm.nix          # Developer VM (user, rustup, aliases)
│   └── plasma.nix          # Plasma-only target profile
├── crates/                 # Rust source
│   ├── agnt-common/        # Shared types (audit, config, memory, models)
│   ├── agntctl/            # OS control CLI (11 tools)
│   └── agntd/              # LLM-powered agent daemon
├── docs/                   # Documentation, banners, screenshots
├── .specs/                 # Spec-driven project memory
│   ├── project/            # Vision, roadmap, state
│   └── features/           # Feature specs, design, tasks
└── Makefile                # Dev shortcuts
```

## Crate architecture

| Crate | Depends on | Purpose |
|---|---|---|
| `agnt-common` | serde, chrono | `AuditEntry`, `ConfigProposal`, `CoreMemory`, `ModelsConfig` |
| `agntctl` | agnt-common, clap | CLI tool — all 11 tool implementations |
| `agntd` | agnt-common, tokio, reqwest | LLM agent daemon — prompt assembly, tool dispatch, session store |

## NixOS modules

| Module | Purpose |
|---|---|
| `base.nix` | `agntos.enable`, configDir, tmpfiles for `/etc/agntos/`, `dirImports` for packages/services/options |
| `desktop-plasma.nix` | SDDM + Plasma 6 Wayland, no other DE support |
| `agent.nix` | systemd user service for agntd (socket mode, packaged path) |
| `vm.nix` | QEMU settings, shared folder, SSH forwarding, graphics |

## Adding a new tool

1. Implement the logic in `crates/agntctl/src/` (new file or existing module)
2. Add the CLI subcommand in `crates/agntctl/src/main.rs`
3. Add the LLM tool definition in `crates/agntd/src/llm.rs` (`tool_definitions()`)
4. Wire the tool execution in `crates/agntd/src/agent.rs` (`execute_tool_call()`)
5. Add tests
6. Run `cargo test`
7. Run the eval runbook in the VM

## Conventions

- **No comments in code** unless the logic is non-obvious. The code should be self-documenting.
- **Tests live next to the code** (Rust `#[cfg(test)] mod tests` convention).
- **Every mutation is audited.** If a new tool changes system state, it must write to `audit.jsonl`.
- **Proposals are JSON** at `/etc/agntos/proposals/<id>.json`. Apply reads from there, then cleans up.
- **Nix files are generated**, not hand-edited. The agent writes them via `propose` templates.
