# Kirigami Settings — Task Tracking

ID prefix: KS

## Phase 3 v1 (Complete)

| Task | ID | Status | Verification |
|---|---|---|---|
| Shared wire protocol types | KST-01 | [x] | 6 tests, 12 total in agnt_common |
| agntd persistent session handler | KST-02 | [x] | 5 tests, 27 total in agntd |
| agntos-settings crate scaffold | KST-03 | [x] | Compiles, 0 dependency warnings |
| Socket connection and reconnect loop | KST-04 | [x] | 4 session tests, exponential backoff verified |
| Data models (Chat/Proposal/Status/Audit) | KST-05 | [x] | 16 model tests, 24 total in agntos-settings |
| QML Chat page with tool cards | KST-06 | [x] | main.qml + 5 components created |
| QML Status, Proposals, Activity pages | KST-07 | [x] | 3 pages + AuditEntryCard created |
| Integration test (mock server) | KST-08 | [x] | 6 integration tests, 30 total in agntos-settings |
| NixOS package and module | KST-09 | [x] | nix build succeeds, .desktop created |
| Documentation and spec update | KST-10 | [x] | ROADMAP, STATE, AGENTS.md updated |

## Phase 3.2 (Planned)

| Task | ID | Status | Priority |
|---|---|---|---|
| Model routing configuration page | KST-11 | [ ] | High |
| Memory viewer/editor page | KST-12 | [ ] | Medium |
| cxx-qt or qmetaobject Rust↔QML bridge | KST-13 | [ ] | High |
| Watchdog event wiring (push to GUI) | KST-14 | [ ] | Medium |
| Multiple simultaneous GUI connections | KST-15 | [ ] | Low |

## Current Test Count

- agnt-common: 12 tests
- agntd: 27 tests
- agntos-settings: 30 tests (24 unit + 6 integration)
- agntctl: 51 tests
- **Total: 120 tests**
