# Baseline Clean — Tasks

**Feature:** `baseline-clean`  
**Execute order:** top to bottom; `[P]` = parallelizable after dependencies.

---

## Phase 0 — Safety

### T0.1 Tag legacy snapshot
- **What:** `git tag pre-wedge-a-legacy` on current HEAD (user confirms push policy).
- **Done when:** Tag exists locally; user informed to `git push origin pre-wedge-a-legacy` if desired.
- **Gate:** `git tag -l 'pre-wedge-a-legacy'`

---

## Phase 1 — Policy & docs (parallel)

### T1.1 Add GPL-3.0-or-later LICENSE `[P]`
- **What:** Root `LICENSE` (GPL-3.0-or-later text); update workspace `Cargo.toml` `license`.
- **Where:** `/LICENSE`, `/Cargo.toml`, per-crate `Cargo.toml` if needed.
- **Done when:** `rg 'license = "MIT"'` only in legacy or none in core crates.
- **Gate:** `cargo check`

### T1.2 Commit and align VISION + STATE `[P]`
- **What:** Stage `.specs/project/VISION.md`; ensure STATE points to `baseline-clean` then `wedge-a`.
- **Done when:** Files committed or ready; no “Settings Stabilization” as current phase.
- **Gate:** manual review

### T1.3 FROZEN markers on parked specs `[P]`
- **What:** Add `FROZEN.md` in `pi-tauri-migration`, `cc-gui-v2`, `kirigami-settings`, `settings-stabilization`, `shadcn-ui-migration` pointing to VISION.
- **Done when:** Each dir has pointer; one line in spec headers optional.
- **Gate:** `ls .specs/features/*/FROZEN.md`

### T1.4 README + AGENTS alignment `[P]`
- **What:** Short README rewrite: module-first, general agent, portfolio, link VISION; AGENTS points to VISION first (partially done).
- **Done when:** No “Kirigami production” without “parked”.
- **Gate:** manual review

---

## Phase 2 — Rust workspace

### T2.1 Remove agntos-cc from workspace
- **Depends:** T0.1
- **What:** Remove `agntos-cc` from root `Cargo.toml` members; move `crates/agntos-cc` → `legacy/agntos-cc` (or delete if user prefers).
- **Done when:** `cargo test` only builds three crates.
- **Gate:** `cargo test`

### T2.2 Relocate agntos-settings
- **Depends:** T0.1
- **What:** Move `crates/agntos-settings` → `legacy/agntos-settings`; add `legacy/README.md`.
- **Done when:** Not in workspace; documented as frozen.
- **Gate:** `test ! -d crates/agntos-settings`

---

## Phase 3 — Nix flake

### T3.1 Trim flake package outputs
- **Depends:** T2.1
- **What:** Remove `agntos-cc`, `agntos-cc-frontend`, `pi-coding-agent`, KDE-only pkgs from default `packages` set; keep agntctl, agntd, branding only if dev-vm still needs them until wedge-a.
- **Where:** `flake.nix`, `pkgs/`
- **Done when:** `nix build .#agntctl .#agntd` succeeds.
- **Gate:** `nix build .#agntctl .#agntd`

### T3.2 Remove agntos-cc from dev-vm module flags
- **Depends:** T3.1
- **What:** `agntos.agntos-cc.enable = false` or remove; drop CC from `environment.systemPackages`.
- **Where:** `profiles/dev-vm.nix`, `modules/agntos/agntos-cc.nix` (stop importing or delete module import from base).
- **Done when:** `nix build .#nixosConfigurations.agntos-dev-vm.config.system.build.vm` succeeds (or documented flake check).
- **Gate:** nix build VM (may be slow)

---

## Phase 4 — Hygiene

### T4.1 .gitignore audit `[P]`
- **What:** Ensure `target/`, `node_modules/`, `result`, `crates/agntos-cc/frontend/dist` ignored.
- **Done when:** `git status` clean after build attempt.
- **Gate:** `git check-ignore -v target`

### T4.2 Create empty wedge-a feature scaffold `[P]`
- **What:** `.specs/features/wedge-a/spec.md` placeholder “blocked on baseline-clean”.
- **Done when:** Directory exists for next specify pass.

---

## Verification (feature complete)

- [ ] BCL-001 … BCL-009 satisfied (see spec.md)
- [ ] `cargo test` pass
- [ ] `bash .specs/features/agntos-foundation/eval-runbook.sh` in VM (wedge-a; optional smoke after baseline)
- [ ] Repo size on disk without `target/` documented (optional `du -sh`)
