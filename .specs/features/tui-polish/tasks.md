# TUI Polish Tasks

## TP-WIRE — Protocol + agntd streaming

| Task | Done when |
|------|-----------|
| T1 | `TokenChannel` in `wire.rs` + tests |
| T2 | `llm.rs` emits thinking vs content tokens separately |
| T3 | `socket.rs` parses channel; `ToolCall` includes args |

## TP-TUI — Rendering + interaction

| Task | Done when |
|------|-----------|
| T4 | `markdown.rs` + unit test for bold/code |
| T5 | Live streaming + thinking buffers in draw |
| T6 | Scroll Home + footer hint when not at tail |
| T7 | Tool cards show pretty args |

## TP-DOC — Traceability

| Task | Done when |
|------|-----------|
| T8 | wedge-a spec WA-014 done; STATE updated |

## Gate

```bash
cargo test
cargo build -p agnt
```
