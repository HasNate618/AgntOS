# Kirigami Settings — Design Document

## Architecture Overview

```
┌─────────────────────────────────────────────────────┐
│                    agntos-settings                    │
│                   (QML + Rust/cxx-qt)                │
│                                                      │
│  ┌─────────┐  ┌─────────┐  ┌──────────┐  ┌──────────┐        │
│  │  Chat    │  │ Status  │  │Proposals │  │ Activity │        │
│  │  Page    │  │  Page   │  │  Page    │  │  Page    │        │
│  └────┬─────┘  └────┬────┘  └────┬─────┘  └────┬─────┘        │
│       │              │            │                   │
│  ┌────┴──────────────┴────────────┴─────┐           │
│  │          Rust Backend (cxx-qt)        │           │
│  │  ┌──────────┐ ┌──────────┐ ┌──────┐ │           │
│  │  │  Session  │ │ Protocol │ │Models│ │           │
│  │  │  Machine  │ │  Codec   │ │      │ │           │
│  │  └─────┬─────┘ └─────┬────┘ └──────┘ │           │
│  └────────┼─────────────┼────────────────┘           │
└───────────┼─────────────┼────────────────────────────┘
            │             │
      Unix Domain Socket  │
      /run/agntd/agent.sock
            │
┌───────────┼─────────────────────────────────────────┐
│           ▼          agntd                           │
│  ┌──────────────┐  ┌──────────┐  ┌──────────────┐  │
│  │  Persistent   │  │  Agent   │  │  Watchdog    │  │
│  │  Connection   │  │  Loop    │  │  Thread      │  │
│  │  Handler      │  │          │  │              │  │
│  └──────────────┘  └──────────┘  └──────────────┘  │
│                                                      │
│  ┌──────────┐  ┌──────────┐  ┌──────────────────┐  │
│  │  LLM      │  │  agntctl │  │  /etc/agntos/    │  │
│  │  Client   │  │  calls   │  │  proposals/      │  │
│  │           │  │          │  │  memory/         │  │
│  └──────────┘  └──────────┘  └──────────────────┘  │
└─────────────────────────────────────────────────────┘
```

## Data Flow: Chat Turn

```
User types "install nginx"
        │
        ▼
agntos-settings: send {"type":"chat","prompt":"install nginx"}
        │
        ▼
agntd: create system prompt + user message → LLM
        │
        ▼ (streaming)
agntd → agntos-settings: {"type":"token","content":"I'll"}
agntd → agntos-settings: {"type":"token","content":" install"}
agntos-settings: batch tokens, fade-in over 80ms
        │
        ▼
agntd: LLM requests tool call "inspect" with target="system"
agntd → agntos-settings: {"type":"tool_call","id":"tc-1","name":"inspect","args":{"target":"system"},"status":"running"}
agntos-settings: render spinner card "Inspecting system…"
        │
        ▼
agntd: execute inspect, get result
agntd → agntos-settings: {"type":"tool_result","id":"tc-1","name":"inspect","output":"CPU: 8 cores…","success":true}
agntos-settings: transition spinner → checkmark, collapsible card with output
        │
        ▼ (repeat for propose, then apply)
agntd: LLM requests "apply" for proposal p-abc123
agntd → agntos-settings: {"type":"approval_request","proposal_id":"p-abc123","summary":"Install nginx","tool_call_id":"tc-3"}
agntos-settings: render approval card with Approve/Reject buttons
        │
        ▼ (user taps Approve)
agntos-settings: send {"type":"approve","proposal_id":"p-abc123"}
        │
        ▼
agntd: execute apply, feed result to LLM
agntd → agntos-settings: {"type":"tool_result","id":"tc-3","name":"apply","output":"Applied successfully","success":true}
agntd → agntos-settings: {"type":"token","content":"Done! Nginx is now installed."}
agntd → agntos-settings: {"type":"turn_complete","content":"Done! Nginx is now installed."}
```

## Data Flow: Push Events

```
Watchdog detects disk > 95%
        │
        ▼
agntd: broadcast to all persistent clients
agntd → agntos-settings: {"type":"event","event":"watchdog_alert","data":{"check":"disk","severity":"warning","usage_pct":96}}
agntos-settings: update Status page watchdog card, show notification

agntctl propose creates new proposal
        │
        ▼
agntd: inotify or poll detects new file in proposals/
agntd → agntos-settings: {"type":"event","event":"proposal_created","data":{"proposal_id":"p-new","summary":"Install htop"}}
agntos-settings: add to Proposals page, badge count on tab

New audit entry written
        │
        ▼
agntd: broadcast audit_entry event
agntd → agntos-settings: {"type":"event","event":"audit_entry","data":{"id":"a-18af76","action":{"type":"Apply","proposal_id":"p-abc123"},"summary":"Applied: Install nginx","result":{"status":"Success"},"timestamp":"2025-05-16T14:30:00Z"}}
agntos-settings: prepend to Activity page timeline
```

## Session State Machine

```
Disconnected ──connect()──► InitSent ──session_ready──► Ready
     ▲                                                    │
     │                                            chat() │ │ turn_complete
     │                                                   ▼ │
     │                                              Chatting │
     │                                                   │   │
     │                      approval_request             │   │
     │                                                   ▼   │
     │                                              Awaiting │
     │                                               Approval │
     │                                                   │
     │                          approve/dismiss ──────────┘
     │
     └────── socket error / timeout ─── reconnect loop
```

## Component Responsibilities

### `backend/protocol.rs`
- Serde structs for all message types (init, chat, token, tool_call, tool_result, approval_request, event, etc.)
- `serialize()` and `deserialize()` functions for NDJSON line protocol
- `MessageType` enum for dispatch

### `backend/session.rs`
- State machine: Disconnected → InitSent → Ready → Chatting → AwaitingApproval
- Handles transitions and enforces protocol correctness
- Reconnect logic with exponential backoff (1s → 2s → 4s → … → 30s max)
- On reconnect: re-send `init`, re-read proposals from disk

### `backend/mod.rs`
- Owns the Unix socket connection
- Read loop: reads lines, deserializes, dispatches to session
- Write loop: serializes messages from Q_INVOKABLE calls, writes to socket
- Propagates connection state to QML via `connected` property

### `models/chat_model.rs`
- QAbstractListModel with roles: type, content, toolName, toolArgs, toolOutput, toolSuccess, proposalId, proposalSummary, timestamp
- Each entry is a chat row: user message, assistant text, tool_call card, tool_result card, or approval card
- `appendToken()`: appends to the current assistant message (batched)
- `addToolCall()`: inserts a running-tool card
- `resolveToolCall()`: transitions spinner → result
- `addApprovalRequest()`: inserts approval card

### `models/proposal_model.rs`
- QAbstractListModel with roles: proposalId, summary, createdAt, status
- `refresh()`: reads `/etc/agntos/proposals/*.json` and updates model
- `apply()`: sends `approve` message
- `dismiss()`: sends `dismiss` message
- Receives `proposal_created` events to add new entries

### `models/status_model.rs`
- QObject with Q_PROPERTY: connected, profileName, modelName, endpoint, cpuInfo, ramUsed, diskUsed, failedUnits, watchdogInterval, watchdogDiskThreshold, watchdogAlertCount, lastCheckTime
- `refresh()`: sends `{"type": "status", "target": "system"}` request, receives `status_response` with structured data
- Updated by push events for watchdog data

### `models/audit_model.rs`
- QAbstractListModel with roles: auditId, timestamp, actionType, summary, status, actor, prompt, rationale, filesChanged, rollbackHint, resultMessage
- `load(count)`: sends `audit` request with `action: "list"`, receives `audit_response`
- `search(query)`: sends `audit` request with `action: "search"`, receives `audit_response`
- `show(id)`: sends `audit` request with `action: "show"`, receives `audit_response` with single entry
- `rollback(id)`: sends `chat` message: "rollback audit entry {id}"
- Receives `audit_entry` push events to prepend new entries without full refresh

## Chat UX Details

### Fade-In Streaming

QML implementation:

```qml
// Token text is appended in batches. Each batch gets an opacity animation.
Text {
    id: messageText
    text: messageContent  // updated by model.appendToken()
    opacity: 0

    PropertyAnimation on opacity {
        from: 0
        to: 1
        duration: 80
        easing.type: Easing.OutQuad
    }

    onTextChanged: opacityAnimation.restart()
}
```

Rust side batches tokens on a 30ms timer before emitting a single `dataChanged` signal covering the appended range.

### Tool Call Card States

```
State: Running
┌─ 🔍 Inspecting system… ────────┐
│  [░░░ spinner ░░░]               │
└──────────────────────────────────┘

State: Done (success)
┌─ ✓ Inspected system ────────────┐
│  CPU: 8 cores, RAM: 32GB       │  ← truncated to 5 lines
│  [Show more]                    │  ← tap to expand
└──────────────────────────────────┘

State: Done (error)
┌─ ✗ Inspected system ────────────┐
│  TOOL_ERROR: endpoint timeout  │
└──────────────────────────────────┘
```

### Approval Card States

```
State: Pending
┌─ ⚠️ Apply proposal p-abc123? ─────┐
│  Install nginx for web serving     │
│  [✓ Approve]     [✗ Reject]       │
└─────────────────────────────────────┘

State: Approved (applying)
┌─ ⚠️ Applying p-abc123… ───────────┐
│  [░░░ spinner ░░░]                 │
└─────────────────────────────────────┘

State: Applied
┌─ ✓ p-abc123 applied ───────────────┐
│  Nginx installed, rebuild ok       │
└─────────────────────────────────────┘

State: Rejected
┌─ ✗ p-abc123 rejected ──────────────┐
│  User dismissed                    │
└─────────────────────────────────────┘
```

## agntd Protocol Extension

### Changes to `main.rs`

The socket mode handler currently does:
1. Accept connection → read `{"prompt": "..."}` → respond → close.

New persistent mode:
1. Accept connection → read first message.
2. If `{"prompt": "..."}` → existing one-shot path, respond, close.
3. If `{"type": "init", ...}` → enter persistent mode:
   a. Bootstrap LLM session (reuse existing `init_llm_session()` logic).
   b. Send `session_ready` with profile/model/pending proposals.
   c. Enter read loop: parse incoming NDJSON messages, dispatch by type.
   d. On `chat`: start agent turn, stream `token` messages.
   e. On `approve`/`dismiss`: resolve pending approval, continue agent turn.
   f. On `cancel`: abort current agent turn.
   g. On disconnect: clean up session, remove from broadcast list.

### Approval Handling

Currently `apply` and `rollback` use `util::confirm()` which reads stdin. In persistent mode:
- Replace `util::confirm()` with a callback-based approval system.
- When an approval-gated tool is called, the agent loop emits `approval_request` and suspends.
- The GUI sends `approve` or `dismiss`.
- The agent loop resumes with the approval result.

Implementation: `ApprovalGate` enum in `agent.rs`:
```rust
enum ApprovalGate {
    None,
    Pending { proposal_id: String, tool_call_id: String, summary: String },
}
```

When `ApprovalGate::Pending`, the agent turn yields control. The persistent connection handler resolves the gate and resumes the turn.

### Event Broadcasting

Add a `broadcast` channel (`tokio::sync::broadcast`) to `DaemonBootstrap`:
- Watchdog: after triage, broadcast `watchdog_alert`.
- Proposal detection: watch `/etc/agntos/proposals/` with inotify or poll, broadcast `proposal_created`.
- Rebuild status: broadcast `rebuild_started`/`rebuild_complete` during `apply`.
- All persistent connections subscribe to the broadcast channel and forward events as NDJSON messages.

## Dependencies

### Cargo (Rust)

- `cxx-qt` + `cxx-qt-build` — Rust/QML bridge
- `serde` + `serde_json` — NDJSON protocol
- `tokio` — async runtime (already a dependency of agntd)
- `serde` — already used throughout

### Nix / Qt

- `qt6.qtdeclarative` — QML runtime
- `kdePackages.kirigami` — Kirigami components
- `kdePackages.qtquickcontrols2` — Qt Quick Controls styling
- The app links against Qt6 and Kirigami via cxx-qt's CMake integration

### Build

- `agntos-settings` enters the Cargo workspace
- Nix flake gets `packages.agntos-settings` via `crane` or `naersk`
- `modules/agntos/agntos-settings.nix` provides the NixOS module