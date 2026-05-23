# Feature Spec: Baseline Clean Repository

**Feature ID:** `baseline-clean`  
**Scope:** Large (repo surgery + policy; precedes wedge A implementation)  
**Authority:** Implements [.specs/project/VISION.md](../../project/VISION.md) decisions D-01–D-20.

## Problem Statement

The repository accumulated parallel product paths (Kirigami settings, Tauri/Pi Control Centre, Plasma branding, Node frontends) that contradict the current vision: a **module-first**, **Rust core**, **terminal-first** agent on NixOS for portfolio and learning. The tree is hard to navigate (~20GB with build artifacts), specs contradict each other, and agents default to the wrong stack.

We need a **clean baseline** that contains only what wedge A requires, with frozen legacy work clearly archived or removed, before building `agnt` TUI and the Cage/Foot/tmux dev edition.

## Goals

- [ ] A new contributor (or agent) can understand the product from `VISION.md` + `README.md` in under 10 minutes.
- [ ] Rust workspace builds and tests only the **core** crates (`agnt-common`, `agntctl`, `agntd`).
- [ ] Nix flake evaluates and builds **dev VM** without pulling CC/Pi/KDE as required dependencies.
- [ ] GPL-3.0-or-later is declared in a root `LICENSE` and workspace metadata.
- [ ] Legacy GUI/Pi work is **not** on the default build path (removed or moved under `legacy/` with no flake reference).

## Out of Scope

| Item | Reason |
|------|--------|
| `agnt` TUI implementation | Next feature (`wedge-a`) |
| Cage/Foot/tmux dev profile | Next feature (`wedge-a`) |
| Memory path migration to XDG state | Next feature (`wedge-a`) |
| Remove `apply` from LLM tools | Next feature (`wedge-a`) |
| Home Manager | Post-v1 per VISION |
| ISO / graphical installer | Later |
| MCP, skills loader | v1.1+ |

## User Stories

### P1: Core-only Rust workspace ⭐ MVP

**User Story:** As the maintainer, I want the Cargo workspace to contain only core crates so `cargo test` reflects what we still ship.

**Why P1:** Everything else depends on a honest build surface.

**Acceptance Criteria:**

1. WHEN `cargo test` runs at repo root THEN workspace SHALL include only `agnt-common`, `agntctl`, `agntd` (no `agntos-cc` member).
2. WHEN `cargo test` completes THEN all workspace tests SHALL pass (current baseline: ~93 tests).
3. WHEN a developer opens `Cargo.toml` THEN they SHALL NOT see `agntos-settings` as a workspace member.

**Independent Test:** `cargo test` green on a clean checkout without `crates/agntos-cc/frontend/node_modules`.

---

### P2: Legacy code isolated ⭐ MVP

**User Story:** As the maintainer, I want parked GUI/Pi code out of the default tree so agents and Nix do not pick it up accidentally.

**Why P1:** Prevents repeat of “vibecoded mess” paths.

**Acceptance Criteria:**

1. WHEN browsing `crates/` THEN `agntos-cc` and `agntos-settings` SHALL be absent OR live only under `legacy/` with a `legacy/README.md` explaining they are frozen.
2. WHEN reading root `flake.nix` THEN default `packages` and `agntos-dev-vm` SHALL NOT require `agntos-cc`, `pi-coding-agent`, or KDE theme packages.
3. WHEN an agent reads `AGENTS.md` THEN it SHALL point to `VISION.md` and state CC/Kirigami are parked.

**Independent Test:** `nix flake check` / dev VM eval succeeds without CC package in closure (profile may still reference Plasma until wedge-a — see P3).

---

### P3: Specs and docs aligned with vision ⭐ MVP

**User Story:** As a portfolio reviewer, I want one canonical direction document and no conflicting “current phase” claims.

**Acceptance Criteria:**

1. WHEN opening `.specs/project/` THEN `VISION.md` SHALL exist and `STATE.md` SHALL reference wedge A (not Settings Stabilization as current).
2. WHEN opening frozen feature dirs (`pi-tauri-migration`, `cc-gui-v2`, etc.) THEN each SHALL contain a one-line `FROZEN.md` or header pointing to VISION.
3. WHEN opening `README.md` THEN the intro SHALL match VISION (module-first, general agent, portfolio honest) within one paragraph of drift.

**Independent Test:** Read PROJECT vs VISION — no contradictory “GUI is production path” without FROZEN label.

---

### P4: License and hygiene ⭐ MVP

**User Story:** As the author, I want GPL and clean git hygiene before public portfolio push.

**Acceptance Criteria:**

1. WHEN inspecting repo root THEN `LICENSE` SHALL exist (GPL-3.0-or-later).
2. WHEN inspecting `Cargo.toml` workspace `license` field THEN it SHALL be `GPL-3.0-or-later` (or per-crate equivalent).
3. WHEN running `git status` on a fresh clone after build THEN `target/` and `**/node_modules/` SHALL be ignored and not tracked.

**Independent Test:** `git check-ignore target` succeeds; no `node_modules` in index.

---

### P2: Slim Nix package set (optional in baseline if low risk)

**User Story:** As a maintainer, I want `pkgs/` to list only packages the core flake still builds by default.

**Acceptance Criteria:**

1. WHEN listing `packages.x86_64-linux` in flake THEN default set SHALL be `agntctl`, `agntd`, `agnt-common` (if exposed), and VM-related essentials — not `agntos-cc` unless behind an optional flake output or `legacy` comment.

**Independent Test:** Documented list in `design.md` matches flake after edit.

---

## Requirements (traceable)

| ID | Requirement | Priority |
|----|-------------|----------|
| BCL-001 | Workspace members = agnt-common, agntctl, agntd only | P1 |
| BCL-002 | `cargo test` passes | P1 |
| BCL-003 | Legacy crates not in default `crates/` or under `legacy/` with README | P1 |
| BCL-004 | Flake default packages exclude agntos-cc and pi-coding-agent | P1 |
| BCL-005 | VISION.md committed; STATE current phase = baseline-clean → wedge-a | P1 |
| BCL-006 | Frozen markers on obsolete feature specs | P1 |
| BCL-007 | GPL-3.0-or-later LICENSE + Cargo license field | P1 |
| BCL-008 | .gitignore covers target, node_modules, large artifacts | P1 |
| BCL-009 | README aligned with VISION | P1 |
| BCL-010 | Optional: archive branch or tag `pre-wedge-a-legacy` before deletion | P2 |

## Success Metrics

- Clone → `cargo test` < 2 min on typical dev machine (no 300MB frontend).
- `nix build .#agntctl` and `nix build .#agntd` succeed.
- Zero references to “production path = agntos-cc” in non-FROZEN docs.

## Dependencies

- Decisions in `VISION.md` are accepted (no reopening Plasma-as-v1).
- Maintainer accepts git history may use a tag/branch before deleting large subtrees.
