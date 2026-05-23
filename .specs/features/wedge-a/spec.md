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
| WA-012 | `agnt` socket client (connect to `agntd`, REPL fallback) | Done |
| WA-010 | Ratatui TUI (streaming, tool cards, slash commands) | Done |
| WA-011 | TUI apply/rollback approval overlay (`y`/`n`) | Done |

## Deferred (v1.1 / polish)

| ID | Item |
|----|------|
| WA-013 | TUI: `/audit`, session list, read-only SKILL.md loader |
| WA-014 | Richer tool-arg display in tool cards |

## Verification

- `cargo test` — workspace green
- `nix build .#agnt .#agntctl .#agntd`
- VM: `agnt`, `agnt system inspect`, propose + apply flow
