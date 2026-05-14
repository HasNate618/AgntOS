# AgntOS

AI-native Linux distribution based on NixOS.

## How the Flake Works

The `flake.nix` defines everything about AgntOS in one Nix expression:

```
flake.nix
├── inputs              → github:NixOS/nixpkgs/nixos-24.11
├── nixosConfigurations
│   ├── agntos-dev-vm   → Dev VM (Plasma + developer tools + QEMU)
│   └── agntos-plasma   → Target system profile (Plasma only)
└── packages
    ├── agntctl         → OS control tool (Rust, placeholder)
    └── agntd           → Agent daemon (Rust, placeholder)
```

Each configuration is built by composing NixOS modules:

```
agntos-dev-vm = nixpkgs.lib.nixosSystem {
  modules = [
    modules/agntos/base.nix          # agntos.enable, configDir
    modules/agntos/desktop-plasma.nix # SDDM + Plasma 6 Wayland
    modules/agntos/vm.nix            # QEMU settings + shared folder
    profiles/dev-vm.nix              # developer user, rustup, aliases
  ];
};
```

The flake produces a **VM runner** at:
`nixosConfigurations.agntos-dev-vm.config.system.build.vm`

This generates a shell script that starts QEMU with the fully configured system (kernel, initrd, disk image, Plasma, users, packages, shared folders, and SSH forwarding).

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

First boot: VM window opens QEMU with a NixOS disk image, boots into Plasma/SDDM login.

### Login

- **User:** `developer` (password: `agntos`)
- **Root:** password: `agntos`

### SSH

```bash
ssh -p 2222 developer@localhost
```

The VM forwards host port 2222 → guest port 22.

### Shared Folder

When `PRJ_ROOT` is set at build time, the repo is shared into the VM at `/mnt/agntos-src`. Edit on host, build inside VM.

### Rebuild

To update the VM config (add packages, change desktop, etc.):

```bash
# Rebuild on host
nix build --impure .#nixosConfigurations.agntos-dev-vm.config.system.build.vm
# Stop the VM, restart (the old disk won't be affected, only the system closure)
```

To pick up changes inside the running VM:

```bash
# Inside VM, if you modified the flake:
sudo nixos-rebuild switch --flake /mnt/agntos-src
# Or just rebuild on host and reboot the VM
```

### Stop

```bash
# In the VM window: Ctrl+Alt+G then close window
# Or from host:
pkill qemu-system-x86_64
```

## Development

### Inside the VM

```bash
# The shared folder auto-mounts at /mnt/agntos-src
agntos          → cd /mnt/agntos-src
agnt-build      → cargo build --release
agnt-check      → cargo check
agnt-inspect    → cargo run --bin agntctl -- inspect
agnt-agent      → cargo run --bin agntd
```

### Outside the VM

```bash
# Build Rust tools on host (faster, no Nix needed):
cargo check
cargo build

# Build the full NixOS system (requires Nix):
export PRJ_ROOT=$(pwd)
nix build --impure .#nixosConfigurations.agntos-dev-vm.config.system.build.vm
```

## Project Structure

```
agntos/
├── flake.nix               # Nix entry point
├── Cargo.toml              # Rust workspace root
├── modules/agntos/         # NixOS modules
│   ├── base.nix            # AgntOS base (enable, configDir, edition)
│   ├── desktop-plasma.nix  # Plasma 6 Wayland desktop
│   └── vm.nix              # QEMU VM settings + shared folder
├── profiles/               # System profiles
│   ├── dev-vm.nix          # Developer VM (user, rustup, aliases)
│   └── plasma.nix          # Plasma-only target profile
├── pkgs/                   # Nix package definitions
│   ├── agntctl/default.nix
│   └── agntd/default.nix
├── crates/                 # Rust source
│   ├── agnt-common/        # Shared types (audit, config, proposals)
│   ├── agntctl/            # OS control CLI tool
│   └── agntd/              # Agent daemon (Hermes-like)
├── docs/
│   └── dev-vm-vbox.md      # VirtualBox setup (previous approach)
├── scripts/
│   └── create-vbox-vm.sh   # VBox VM creation (previous approach)
├── .specs/                 # Spec-driven project docs
│   ├── project/            # Vision, roadmap, state
│   └── features/           # Feature specs, design, tasks
└── Makefile                # Dev shortcuts (make check, make build)
```
