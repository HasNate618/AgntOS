# Wedge A Specification

**Authority:** [.specs/project/VISION.md](../../project/VISION.md)  
**Depends on:** [baseline-clean](../baseline-clean/spec.md) (complete)

## Goal

One credible path: boot dev VM → `agnt` → chat → propose → human/policy apply → audit → rollback; agent state survives `nixos-rebuild --rollback`.

## Delivered (this pass)

| ID | Requirement | Status |
|----|-------------|--------|
| WA-001 | Memory + sessions under XDG state dir | Done |
| WA-002 | LLM tools exclude `apply` / `rollback` | Done |
| WA-003 | `settings.json` `auto_apply` (dev VM = auto) | Done |
| WA-004 | Unified `agnt` binary (`chat`, `daemon`, `system`) | Done |
| WA-005 | Dev VM: Cage + Foot + tmux (no Plasma default) | Done |

## Deferred (v1.1 / polish)

| ID | Item |
|----|------|
| WA-010 | Full ratatui TUI (streaming, tool cards, slash commands) |
| WA-011 | TUI apply/rollback approval UX |
| WA-012 | `agnt` socket client (today: foreground `agntd` REPL) |

## Verification

- `cargo test` — workspace green
- `nix build .#agnt .#agntctl .#agntd`
- VM: `agnt`, `agnt system inspect`, propose + apply flow
