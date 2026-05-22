# CC GUI v2 Tasks

| ID | Task | Status |
|----|------|--------|
| T1 | Spec + supersede docs | done |
| T2 | Fix text_delta → text; propose permissions (Nix group + env) | done |
| T3 | Logo in UI + refresh app icons from brand PNG | done |
| T4 | `list_sessions`, models Tauri commands | done |
| T5 | ThreadList sidebar + ModelSelector in composer | done |
| T6 | ModelsPage (profiles CRUD) | done |
| T7 | IPC audit + `cargo check` / `npm run build` | done |

## Verification

```bash
cargo check -p agntos-cc
cd crates/agntos-cc/frontend && npm run build
```

VM: propose from chat creates file in `/etc/agntos/proposals/` without EACCES.
