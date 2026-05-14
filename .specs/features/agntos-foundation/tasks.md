# AgntOS Foundation Tasks

## Task Status

- `[ ]` Pending
- `[~]` In progress
- `[x]` Complete
- `[!]` Blocked

## Tasks

### T001: Create Initial Nix Flake

Status: `[ ]`

Requirements: AGF-001, AGF-002

What:
Create the initial `flake.nix` and a placeholder NixOS system structure for AgntOS.

Where:
- `flake.nix`
- `modules/agntos/base.nix`
- `modules/agntos/desktop-plasma.nix`
- `profiles/dev-vm.nix`

Done when:
- `nix flake check` can evaluate or fails only on explicitly documented missing package implementations.
- The module/profile structure exists.
- Plasma is the only desktop profile.

Verification:
- Run `nix flake check` if Nix is available.
- If Nix is unavailable, run a syntax/structure review and record the limitation.

### T002: Add Dev VM Profile

Status: `[ ]`

Requirements: AGF-003

What:
Create the NixOS dev VM profile with a source-sharing approach for local development.

Where:
- `modules/agntos/vm.nix`
- `profiles/dev-vm.nix`
- `README.md`

Done when:
- The dev VM config builds from Nix.
- The planned host source mount path is documented.
- The VM root remains Nix-built.

Verification:
- Run the VM build command if possible.
- Document the exact command and any host requirements.

### T003: Initialize Rust Workspace

Status: `[ ]`

Requirements: AGF-004

What:
Create the Rust workspace for AgntOS system tools.

Where:
- `Cargo.toml`
- `crates/agnt-common/`
- `crates/agntctl/`
- `crates/agntd/`

Done when:
- `cargo check` succeeds for placeholder crates.
- `agntctl` and `agntd` binaries exist as minimal stubs.

Verification:
- Run `cargo check`.

### T004: Package Rust Tools In Nix

Status: `[ ]`

Requirements: AGF-001, AGF-004

What:
Expose the Rust binaries as Nix packages.

Where:
- `pkgs/agntctl/default.nix`
- `pkgs/agntd/default.nix`
- `flake.nix`

Done when:
- The flake exposes packages for `agntctl` and `agntd`.
- The dev VM includes the tools.

Verification:
- Run `nix build .#agntctl` and `nix build .#agntd` if possible.

### T005: Define AgntOS Managed Config Tree

Status: `[ ]`

Requirements: AGF-005, AGF-008

What:
Choose and document the v0 path and shape for AgntOS-managed Nix configuration.

Where:
- `docs/config-model.md`
- `modules/agntos/base.nix`
- `.specs/project/STATE.md`

Done when:
- The config path is documented.
- The boundary between AgntOS-managed config and arbitrary user config is explicit.
- Open questions around Nix vs TOML/YAML are narrowed or documented.

Verification:
- Review against AGF-005 and AGF-008.

### T006: Implement `agntctl inspect`

Status: `[ ]`

Requirements: AGF-004, AGF-005

What:
Implement the first read-only `agntctl inspect` command.

Where:
- `crates/agntctl/`
- `crates/agnt-common/`

Done when:
- `agntctl inspect system` prints basic OS, kernel, desktop, CPU, memory, and GPU info where available.
- The command does not require elevated privileges for basic info.

Verification:
- Run `cargo test`.
- Run `agntctl inspect system` locally or in VM.

### T007: Implement `agntctl propose`

Status: `[ ]`

Requirements: AGF-005, AGF-006

What:
Implement an initial proposal flow for a safe Nix-backed change.

Where:
- `crates/agntctl/`
- `docs/config-model.md`

Done when:
- A command can generate a planned config change without applying it.
- The proposal includes target files and a human-readable summary.

Verification:
- Run unit tests for proposal generation.
- Manually inspect generated proposal output.

### T008: Implement Audit Log Skeleton

Status: `[ ]`

Requirements: AGF-005, AGF-006

What:
Create the local structured audit log format and basic write/read commands.

Where:
- `crates/agnt-common/`
- `crates/agntctl/`
- `docs/audit-log.md`

Done when:
- OS actions can append audit entries.
- `agntctl audit list` can read entries.
- The log format is documented.

Verification:
- Run unit tests for serialization/deserialization.

### T009: Add Minimal `agntd` Agent Stub

Status: `[ ]`

Requirements: AGF-006

What:
Create a minimal agent daemon stub that can call `agntctl inspect`.

Where:
- `crates/agntd/`
- `modules/agntos/agent.nix`

Done when:
- `agntd` can run as a user/session process.
- It can invoke or link to the OS inspect functionality.
- It is packaged into the dev VM.

Verification:
- Run `cargo check`.
- Run the daemon stub manually.

### T010: Define Model Routing Config

Status: `[ ]`

Requirements: AGF-007

What:
Define the v0 model routing schema and default task classes.

Where:
- `docs/model-routing.md`
- `crates/agnt-common/`
- `.specs/project/STATE.md`

Done when:
- Task classes are documented.
- Provider/model assignment format is documented.
- OpenAI-compatible endpoint support is represented.
- Local backend support is represented as an extension point.

Verification:
- Review against AGF-007.

### T011: Create Minimal Kirigami Direction Doc

Status: `[ ]`

Requirements: AGF-002, AGF-006, AGF-007

What:
Document the first Kirigami UI scope without blocking lower-level agent work.

Where:
- `docs/ui.md`

Done when:
- The first UI surfaces are listed.
- CLI/dev interface fallback is explicitly allowed.
- Plasma-only scope is restated.

Verification:
- Review against product principles in `PROJECT.md`.

### T012: Build First End-To-End Demo

Status: `[ ]`

Requirements: AGF-001 through AGF-009

What:
Connect the dev VM, `agntctl inspect`, `agntd`, and initial model routing/config stubs into a small demonstration.

Where:
- Entire foundation stack.

Done when:
- AgntOS boots in a dev VM.
- A user can run an early assistant or CLI.
- The assistant/tool can inspect the OS.
- The project can explain the next safe config-change path.

Verification:
- Run VM smoke test.
- Record manual test notes.
