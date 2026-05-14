# AgntOS

AI-native Linux distribution based on NixOS.

## Development

### Prerequisites

- Nix (with flakes enabled)
- Rust toolchain
- QEMU (for `nixos-rebuild build-vm`)
- Or: VirtualBox (alternative dev VM path)

### Quick Start (Nix/QEMU)

```bash
# Build and run the dev VM:
nix build .#nixosConfigurations.agntos-dev-vm.config.system.build.vm
./result/bin/run-agntos-dev-vm-vm
```

To share the local source into the VM for fast iteration, set `PRJ_ROOT`:

```bash
export PRJ_ROOT=$(pwd)
nix build .#nixosConfigurations.agntos-dev-vm.config.system.build.vm
./result/bin/run-agntos-dev-vm-vm
```

The VM mounts the repo at `/mnt/agntos-src`.

### Quick Start (VirtualBox)

See [docs/dev-vm-vbox.md](docs/dev-vm-vbox.md) for VirtualBox setup instructions.

### Building Rust Tools

```bash
cargo build

# Run agntctl:
cargo run --bin agntctl -- inspect

# Run agntd:
cargo run --bin agntd
```

### Project Structure

```
agntos/
  flake.nix          # Nix flake entry point
  Cargo.toml         # Rust workspace root
  modules/agntos/    # NixOS modules
    base.nix         # AgntOS base module
    desktop-plasma.nix  # Plasma desktop defaults
    vm.nix           # Dev VM configuration
  profiles/          # System profiles
    dev-vm.nix       # Developer VM profile
    plasma.nix       # Plasma-only target profile
  pkgs/              # Individual Nix package definitions
  crates/            # Rust source
    agnt-common/     # Shared types (audit log, config, etc.)
    agntctl/         # OS control CLI tool
    agntd/           # Agent daemon
  .specs/            # Spec-driven project docs
    project/         # Vision, roadmap, state
    features/        # Feature specs, design, tasks
```

### Phase 1 (Current)

Goal: Boot into AgntOS and interact with a Hermes-like OS-aware agent.

Active tasks in `.specs/features/agntos-foundation/tasks.md`.
