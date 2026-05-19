# Tasks: Pi + Tauri Migration

## Phase 1: Foundation (AgntOS scaffold + Pi bridge + basic chat)

### PTM-P1-01: Create Tauri v2 project scaffold
- **What**: Initialize `crates/agntos-cc/` as a Tauri v2 app with Svelte frontend. Pure AgntOS branding — no Pi references.
- **Where**: `crates/agntos-cc/`
- **Depends on**: None
- **Verification**: `cargo check -p agntos-cc` passes; `npm run build` in frontend/ succeeds
- **Gate**: `cargo check` + `cargo test`

### PTM-P1-02: Implement Pi bridge (Rust) with identity-stripping flags
- **What**: Build `pi_bridge.rs` that spawns Pi with the correct flags:
  ```
  pi --mode rpc --no-builtin-tools --no-context-files
      --system-prompt <agntos_prompt> --extension <agntos_tools>
  ```
  Reads JSONL events from stdout, writes commands to stdin, routes events as `agent:*` (not `pi:*`), handles lifecycle.
- **Where**: `crates/agntos-cc/src/pi_bridge.rs`
- **Depends on**: PTM-P1-01
- **Verification**: Bridge can spawn Pi, send prompt, receive events, shut down. No Pi branding in any event name or log.
- **Gate**: `cargo test` passes; integration test with Pi binary present

### PTM-P1-03: Create AgntOS system prompt
- **What**: Standalone Markdown file at `/etc/agntos/AGENTS.md` with pure AgntOS instructions. No Pi, Claude, or any third-party references.
- **Where**: `etc/agntos/AGENTS.md` in repo (template)
- **Depends on**: None
- **Verification**: Prompt mentions only agntos_* tools, describes propose→approve→apply workflow, zero Pi mentions.
- **Gate**: Review

### PTM-P1-04: Basic Svelte chat frontend
- **What**: ChatPage.svelte with message list, streaming markdown, text input. Subscribes to `agent:*` Tauri events. No Pi branding.
- **Where**: `crates/agntos-cc/frontend/src/components/ChatPage.svelte`
- **Depends on**: PTM-P1-02
- **Verification**: User can type a message, see streaming response, approve proposals. Status says "Connected" not "Connected to Pi."
- **Gate**: End-to-end in `cargo tauri dev`

### PTM-P1-05: Connection status indicator
- **What**: Status bar showing agent state (connected/disconnected/idle/thinking/streaming), current model. Never mentions Pi.
- **Where**: `crates/agntos-cc/frontend/src/components/StatusIndicator.svelte`
- **Depends on**: PTM-P1-02
- **Verification**: Status updates in real-time. Shows "Connected" not "Connected to Pi."
- **Gate**: Visual verification

### PTM-P1-06: Nix packaging for dev stack
- **What**: Package Pi as hidden dependency, Tauri app, and agntctl. Pi is NOT in user's PATH unless `programs.pi.enable = true`.
- **Where**: `pkgs/agntos-cc/default.nix`, `modules/agntos/`, `flake.nix`
- **Depends on**: PTM-P1-01
- **Verification**: `nix build .#agntos-cc` succeeds. VM includes Tauri app + agntctl. `which pi` fails unless programs.pi enabled.
- **Gate**: `nix build` succeeds

---

## Phase 2: Feature Parity (AgntOS extension + full UI)

### PTM-P2-01: AgntOS Pi extension (agntos-tools)
- **What**: TypeScript Pi extension registering all 10 AgntOS tools: propose, apply, rollback, inspect, audit, memory, bash, read, write, edit. Each shells out to agntctl.
- **Where**: `extensions/agntos-tools/index.ts`
- **Depends on**: PTM-P1-02 (bridge works)
- **Verification**: Pi can call agntos_inspect, agntos_propose, agntos_apply (with confirmation), agntos_bash, etc. Zero Pi default tools visible.
- **Gate**: All 10 agntos_* tools work through Pi

### PTM-P2-02: Approval cards in chat
- **What**: Route extension_ui_request events to frontend as approval cards. User Approve/Reject triggers extension_ui_response.
- **Where**: `src/main.rs` (routing), `frontend/src/components/MessageBubble.svelte`
- **Depends on**: PTM-P2-01, PTM-P1-04
- **Verification**: Agent proposes change, approval card appears, approve triggers agntctl apply, dismiss cancels.
- **Gate**: End-to-end approval flow in VM

### PTM-P2-03: Tool call cards in chat
- **What**: Render agntos_* tool calls with icon, spinner, expandable result. AgntOS-branded styling.
- **Where**: `frontend/src/components/MessageBubble.svelte`
- **Depends on**: PTM-P1-04
- **Verification**: Each tool call shows colored card; running shows spinner; result expandable.
- **Gate**: Visual verification

### PTM-P2-04: Model switching UI
- **What**: Dropdown to switch models. Reads available models from Pi's `get_available_models` RPC, sends `set_model`.
- **Where**: `frontend/src/components/ModelSelector.svelte`
- **Depends on**: PTM-P1-02
- **Verification**: User can switch between configured models mid-session.
- **Gate**: Model switching works

### PTM-P2-05: Proposals page
- **What**: Page showing pending proposals with description, nix changes, Approve/Dismiss actions.
- **Where**: `frontend/src/components/ProposalsPage.svelte`
- **Depends on**: PTM-P2-02
- **Verification**: Proposals appear in list; approve/dismiss triggers agntos_apply/rollback.
- **Gate**: Proposals page functional

### PTM-P2-06: Status page
- **What**: Page showing agent state, model, session info, system info from agntos_inspect.
- **Where**: `frontend/src/components/StatusPage.svelte`
- **Depends on**: PTM-P1-05, PTM-P1-02
- **Verification**: Status page shows real-time agent state, model, system stats.
- **Gate**: Visual verification

### PTM-P2-07: Activity page
- **What**: Page showing audit log from agntctl audit with search/filter.
- **Where**: `frontend/src/components/ActivityPage.svelte`
- **Depends on**: PTM-P2-01
- **Verification**: Activity page shows proposal/apply/rollback history with timestamps.
- **Gate**: Activity page shows correct data

### PTM-P2-08: Session management
- **What**: New session, resume session, session browser in frontend. Uses Pi's new_session, switch_session, get_state RPC.
- **Where**: `crates/agntos-cc/src/pi_bridge.rs` (add commands), `frontend/src/components/SessionBrowser.svelte`
- **Depends on**: PTM-P1-02
- **Verification**: User can start new session, switch between sessions, see session list.
- **Gate**: Session management works in VM

### PTM-P2-09: Pi skills for AgntOS
- **What**: SKILL.md files for agntos-inspect and agntos-bash that teach the agent when to use these tools.
- **Where**: `skills/agntos-inspect/SKILL.md`, `skills/agntos-bash/SKILL.md`
- **Depends on**: None (documents)
- **Verification**: Agent uses inspect when asked about system status.
- **Gate**: Skills work in interactive Pi mode (for testing)

### PTM-P2-10: Memory tool
- **What**: agntos_memory tool reads/writes MEMORY.md and USER.md via agntctl.
- **Where**: `extensions/agntos-tools/index.ts` (add tool)
- **Depends on**: PTM-P2-01
- **Verification**: Agent can remember and recall user preferences across sessions.
- **Gate**: Memory persists across sessions

---

## Phase 3: Polish + Deprecate Old Stack

### PTM-P3-01: Session tree visualization
- **What**: Visual session tree viewer. User can click nodes to resume from that point.
- **Where**: `frontend/src/components/SessionTree.svelte`
- **Depends on**: PTM-P2-08
- **Verification**: User can see branching history, click to resume.
- **Gate**: Tree visualization works

### PTM-P3-02: Compaction settings UI
- **What**: Settings to configure auto-compaction threshold, manual compaction trigger.
- **Where**: `frontend/src/components/SettingsPage.svelte`
- **Depends on**: PTM-P1-02
- **Verification**: User can adjust compaction settings and see effect.
- **Gate**: Settings persist and work

### PTM-P3-03: Remove agntd and agntos-settings from Nix module
- **What**: Remove old stack from Nix module. Only Pi + Tauri remain.
- **Where**: `modules/agntos/`
- **Depends on**: All Phase 2 complete
- **Verification**: VM builds without agntd or agntos-settings; Pi + Tauri work as only stack.
- **Gate**: Full VM test

### PTM-P3-04: Documentation update
- **What**: Update README, AGENTS.md, project docs for new stack.
- **Where**: `README.md`, docs/
- **Depends on**: PTM-P3-03
- **Verification**: New developer can follow docs to set up dev environment.
- **Gate**: Docs review

### PTM-P3-05: Session migration tool
- **What**: Tool to convert old SQLite sessions to Pi JSONL format.
- **Where**: `crates/agntctl/src/`
- **Depends on**: PTM-P2-08
- **Verification**: Old sessions are importable into Pi.
- **Gate**: Migration works

### PTM-P3-06: E2E evaluation
- **What**: End-to-end test: boot VM, start Tauri, chat, propose, approve, apply, verify.
- **Where**: `.specs/features/pi-tauri-migration/eval-runbook.sh`
- **Depends on**: PTM-P3-03
- **Verification**: Full cycle works.
- **Gate**: All eval checks pass

---

## Dependency Graph

```
PTM-P1-01 (Tauri scaffold)
    ├── PTM-P1-02 (Pi bridge with identity-stripping flags)
    │   ├── PTM-P1-04 (Chat frontend)
    │   │   ├── PTM-P2-02 (Approval cards) ── PTM-P2-05 (Proposals page)
    │   │   └── PTM-P2-03 (Tool call cards)
    │   ├── PTM-P1-05 (Status indicator) ── PTM-P2-06 (Status page)
    │   ├── PTM-P2-04 (Model switching)
    │   └── PTM-P2-08 (Session management) ── PTM-P3-01 (Session tree)
    │       └── PTM-P3-05 (Session migration)
    ├── PTM-P1-03 (System prompt)
    └── PTM-P1-06 (Nix packaging) ── PTM-P3-03 (Remove old stack)

PTM-P2-01 (AgntOS Pi extension)
    ├── PTM-P2-02 (Approval flow)
    ├── PTM-P2-07 (Activity page)
    ├── PTM-P2-09 (Pi skills)
    └── PTM-P2-10 (Memory tool)

PTM-P3-02 (Compaction settings) ── PTM-P1-02
PTM-P3-04 (Docs) ── PTM-P3-03
PTM-P3-06 (E2E) ── PTM-P3-03
```

## Risk Register

| Risk | Impact | Mitigation |
|------|--------|------------|
| Pi RPC protocol changes | High | Pin Pi version; abstract RPC layer behind Rust trait |
| Pi --system-prompt doesn't fully strip Pi identity | High | Test with "who are you?" prompt in dev VM |
| agntctl bash/read/write missing features vs Pi built-ins | Medium | Audit Pi's tool capabilities and port missing features to agntctl |
| Node.js dependency size | Medium | Use Nix package; Pi is ~50MB installed |
| User has Pi installed independently, conflicts with AgntOS Pi | Low | Separate config dirs via PI_CODING_AGENT_DIR env var |
| Pi extension API instability | Medium | Pin Pi version; test against specific release |
| --no-builtin-tools breaks Pi internal flows | Low | Verify Pi doesn't depend on its own tools internally |
