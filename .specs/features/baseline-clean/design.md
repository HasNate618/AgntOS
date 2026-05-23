# Baseline Clean — Design

## Target repository shape

After `baseline-clean`, the default tree looks like this:

```text
agntos/
  LICENSE                    # GPL-3.0-or-later
  README.md                  # Aligned with VISION
  AGENTS.md
  Cargo.toml                 # members: agnt-common, agntctl, agntd
  flake.nix
  crates/
    agnt-common/
    agntctl/
    agntd/
  modules/agntos/
    base.nix
    agent.nix
    vm.nix
    desktop-plasma.nix       # kept but NOT imported by dev-vm (wedge-a removes)
  profiles/
    dev-vm.nix               # may still reference plasma until wedge-a (document)
    plasma.nix               # optional profile, not default
  pkgs/
    agntctl/
    agntd/
    # agntos-cc, pi-coding-agent, bart-kde, etc. removed or optional output
  .specs/
    project/  VISION.md, STATE.md, ROADMAP.md, PROJECT.md (synced)
    features/
      baseline-clean/        # this feature
      wedge-a/               # created next, empty until baseline done
      */FROZEN.md on parked features
  legacy/                    # optional: entire parked trees moved here
    README.md
    agntos-cc/
    agntos-settings/
```

## What stays (the “basics”)

| Layer | Keep | Notes |
|-------|------|-------|
| Rust | `agnt-common`, `agntctl`, `agntd` | 93 tests, eval-runbook |
| Nix modules | `base.nix`, `agent.nix`, `vm.nix` | `/etc/agntos` contract unchanged |
| Nix profile | `dev-vm.nix`, `plasma.nix` | Plasma profile exists but dev-vm slimming is wedge-a |
| Specs | VISION, STATE, foundation eval-runbook | Archive noise optional |
| Docs | README, AGENTS | Short honesty about portfolio |

## What goes

| Item | Action |
|------|--------|
| `crates/agntos-cc` | Move to `legacy/` or delete after tag `pre-wedge-a-legacy` |
| `crates/agntos-settings` | Same |
| `pkgs/agntos-cc`, `pkgs/pi-coding-agent`, KDE pkgs | Remove from flake `packages` default set |
| `agntos.agntos-cc.enable` in dev-vm | Remove in wedge-a (baseline may leave if risky) |
| Root `node_modules`, `target/` | Ensure gitignore; never commit |

## Git strategy

1. **Tag** current `main` (or HEAD): `pre-wedge-a-legacy` — recoverable snapshot.
2. Apply baseline-clean on `main` or new branch `wedge-a`.
3. Do **not** rewrite public history if already pushed without user consent.

## Nix evaluation note

Baseline may leave `dev-vm` importing Plasma temporarily if removing it breaks eval in one step. Document in tasks: either baseline only removes Rust/Node packages from flake, wedge-a switches profile to Cage/Foot/tmux. Prefer **baseline = code hygiene**, **wedge-a = product path**.

## Risks

| Risk | Mitigation |
|------|------------|
| Accidental deletion of wanted code | Tag before move; `legacy/` not delete |
| Flake eval fails after package removal | Run `nix flake check` each task |
| README oversells features | Tie every claim to VISION shipped vs planned |
