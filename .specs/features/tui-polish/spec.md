# TUI Polish Specification

**Authority:** [.specs/project/VISION.md](../../project/VISION.md)  
**Depends on:** [wedge-a](../wedge-a/spec.md) (WA-010/011 baseline)  
**Scope:** Large — chat UX parity with modern agent CLIs

## Goal

`agnt` TUI feels like a real agent client: live streaming, visible reasoning/thinking, readable markdown, reliable scroll, richer tool cards.

## Requirements

| ID | Requirement | Priority |
|----|-------------|----------|
| TP-001 | Stream assistant **content** tokens into the chat pane while the turn is in progress (not only after `turn_complete`) | Must |
| TP-002 | Stream **thinking/reasoning** tokens on a separate channel; render dim/italic, distinct from final answer | Must |
| TP-003 | Render assistant and system text with **markdown** subset: headings, bold, italic, inline code, fenced code blocks, lists, links as plain URL | Must |
| TP-004 | Scroll: mouse wheel, PgUp/PgDn, Home (top), End (tail + follow); show indicator when not at bottom | Must |
| TP-005 | **WA-014:** Tool cards show tool name, status, and **formatted args** (pretty JSON, truncated); result preview expandable to more lines | Must |
| TP-006 | Wire protocol backward compatible: `token` without channel defaults to `content` | Must |
| TP-007 | `cargo test` workspace green; existing socket parse tests updated | Must |

## Non-goals (this feature)

- Full CommonMark (tables, images, HTML)
- Syntax highlighting in code blocks
- Split `tui.rs` into many crates (optional refactor, not required)
- GUI / web gateway
- Pi/Tauri embed

## Status

**Implemented** (2026-05): TP-001–007, WA-014.

## Verification

- `cargo test -p agnt -p agnt-common -p agntd`
- Manual: `./dev` → `agnt` → long markdown reply streams; thinking visible when model sends `reasoning` deltas
- Manual: propose flow shows tool args on `propose` tool card
