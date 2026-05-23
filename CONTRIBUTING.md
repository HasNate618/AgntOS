# Contributing

## Quick start

```bash
git clone https://github.com/your-org/agntos
cd agntos
cargo check
cargo test
```

Build the full system (requires Nix):

```bash
export PRJ_ROOT=$(pwd)
nix build --impure .#nixosConfigurations.agntos-dev-vm.config.system.build.vm
```

## Build and run

```bash
# Build Rust tools (no Nix needed):
cargo build --release

# Run the agent:
export AGNTOS_API_KEY=your-key
./target/release/agntd

# Run a specific tool:
./target/release/agntctl inspect system
./target/release/agntctl propose "install htop"
```

## VM workflow

One script handles build, run, SSH, tmux, and eval:

```bash
./scripts/dev-vm.sh              # build (if needed), start VM, attach tmux
./scripts/dev-vm.sh build        # nix build only (sets PRJ_ROOT for /mnt/agntos-src)
./scripts/dev-vm.sh start|stop|status
./scripts/dev-vm.sh ssh          # shell in VM
./scripts/dev-vm.sh tmux         # attach dev tmux (chat / shell / logs)
./scripts/dev-vm.sh eval         # 14-check runbook
./scripts/dev-vm.sh restart --reset-disk   # fresh disk when guest system is stale
```

Login: `developer` / `agntos` on `localhost:2222`.

`--impure` is used automatically because `PRJ_ROOT` enables the 9p virtio mount at `/mnt/agntos-src`.

Inside tmux (or SSH):

```bash
agntos          # cd /mnt/agntos-src
agnt            # chat (socket client → agntd)
agnt system inspect
agnt-tmux       # re-attach dev layout
```

Guest rebuild after flake changes:

```bash
sudo nixos-rebuild switch --flake /mnt/agntos-src#agntos-dev-vm
```

Stop:

```bash
# In the VM window: Ctrl+Alt+G then close
# From host:
pkill qemu-system-x86_64
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

14 checks: general tools, OS-native tools, socket mode, rollback errors.

```bash
ssh -p 2222 developer@localhost
cd /mnt/agntos-src
sudo AGNTOS_CONFIG_DIR=/etc/agntos \
  bash .specs/features/agntos-foundation/eval-runbook.sh
```

## Adding a new tool

1. Implement the logic in `crates/agntctl/src/` (new file or existing module)
2. Add the CLI subcommand in `crates/agntctl/src/main.rs` (clap)
3. Add the LLM tool definition in `crates/agntd/src/llm.rs` (`tool_definitions()`)
4. Wire the tool execution in `crates/agntd/src/agent.rs` (`execute_tool_call()`)
5. Add tests
6. Run `cargo test`
7. Run the eval runbook in the VM

## Conventions

- **No comments in code** unless the logic is non-obvious. Code should be self-documenting.
- **Tests live next to the code** in Rust `#[cfg(test)] mod tests` blocks.
- **Every mutation is audited.** If a new tool changes system state, it must write to `audit.jsonl`.
- **Proposals are JSON** at `/etc/agntos/proposals/<id>.json`. Apply reads from there, then cleans up.
- **Nix files are generated**, not hand-edited. The agent writes them via `propose` templates.
- **Use `#[serde(default)]`** on any new field added to serialized structs (AuditEntry, ConfigProposal) so old serialized data still deserializes.

## Project structure

```
agntos/
├── flake.nix               Nix entry point
├── Cargo.toml              Rust workspace root
├── modules/agntos/         NixOS modules (base, desktop-plasma, agent, vm)
├── profiles/               System profiles (dev-vm, plasma)
├── crates/                 Rust source
│   ├── agnt-common/        Shared types (audit, config, memory, models)
│   ├── agntctl/            OS control CLI (10 tools)
│   └── agntd/              LLM agent daemon
├── docs/                   Documentation, banners
├── .specs/                 Spec-driven project memory
│   ├── project/            Vision, roadmap, state
│   └── features/           Feature specs, design, tasks
└── Makefile                Dev shortcuts
```

## Architecture reference

See `.specs/features/agntos-foundation/design.md` for architecture details and data flow.

## Spec-driven development

Feature work starts in `.specs/features/`. Each feature has:

- `spec.md` -- requirements with traceable IDs
- `design.md` -- architecture, components, data flow
- `tasks.md` -- atomic tasks with verification criteria
- `context.md` -- user decisions (when needed)

See `.specs/project/` for roadmap, vision, and state.
