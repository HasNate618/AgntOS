# Kirigami Settings Experience — Specification

ID prefix: KS

## Goal

A standalone Kirigami application (`agntos-settings`) that serves as the primary user interface for AgntOS. The agent is the interaction model — users chat and the agent uses tools — supplemented by structured dashboard pages for status and proposal management.

## Decisions

| Decision | Choice | Rationale |
|---|---|---|
| App archetype | Standalone Kirigami app | Flexibility, centralizes all AgntOS UI |
| Interaction model | Chat-driven + settings tabs | Agent IS the primary interface; tabs for structured views |
| IPC protocol | Bidirectional NDJSON over persistent Unix socket | Small delta from existing socket mode, streaming + events, no network exposure |
| Tech stack | QML + Rust via cxx-qt | Unified Rust codebase, native Kirigami components |
| Phase 3 pages | Chat, Status, Proposals, Activity | Tight scope; model config and memory deferred |
| Tool call UX | Inline cards with approve/reject buttons | Clear, contextual, no hidden state |
| Event model | Push over persistent connection | Real-time watchdog alerts, proposal notifications |
| Chat text rendering | Fade-in streamed tokens | Clean, not jarring character-by-character |
| Tool progress | Compact loading animation with tool name + short detail | User sees what's happening without wall of text |

## Functional Requirements

### KS-001: Persistent Socket Connection

The GUI maintains a single persistent connection to `/run/agntd/agent.sock`.

- On launch, send `init` message; receive `session_ready`.
- Connection stays open for the lifetime of the app.
- Reconnect with exponential backoff on disconnect.
- Existing one-shot `{"prompt": "..."}` protocol remains supported for non-GUI clients.

### KS-002: Streaming Chat

- User types a message, sends via `chat` message type.
- agntd streams `token` messages; the GUI fades in each token batch (opacity animation over 80ms, batched every 30ms).
- When the agent invokes a tool, a `tool_call` message with `status: "running"` creates an inline card showing the tool name and a spinner.
- When the tool completes, `tool_result` updates the card with the output (truncated to 5 lines, expandable).
- When the agent requests approval (`approval_request`), an inline card renders with summary + Approve/Reject buttons.
- `turn_complete` ends the agent's turn.

### KS-003: Tool Call Cards

Each tool invocation renders as an inline card in the chat stream:

| Tool | Card appearance |
|---|---|
| `inspect` | Collapsible card: "Inspected system" → expand for full output |
| `propose` | Card: "Created proposal p-abc123: Install nginx" |
| `apply` | Approval card: "Apply p-abc123?" with Approve/Reject |
| `rollback` | Approval card: "Roll back to previous generation?" with Approve/Reject |
| `audit` | Collapsible card: "Searched audit log" → expand |
| `memory` | Card: "Memory updated" |
| `read_file` | Collapsible card: "Read /etc/nixos/configuration.nix" → expand |
| `write_file` | Card: "Wrote /etc/agntos/proposals/p-xyz.json" |
| `edit_file` | Card: "Edited /etc/agntos/base.nix" |
| `run_bash` | Collapsible card: "Ran: systemctl status nginx" → expand |

Running tools show:
- Animated spinner (Kirigami `BusyIndicator`)
- Tool name and one-line detail (e.g. "Inspecting system…", "Creating proposal…", "Running: systemctl status nginx")
- Status transitions from `running` → `done` with the spinner replaced by a checkmark or error icon

Completed cards default to collapsed (summary line only). Tap to expand for full output.

### KS-004: Approval Flow

When the agent calls `apply` or `rollback`:

1. agntd sends `approval_request` message (does NOT block waiting for stdin).
2. GUI renders an inline card with the proposal summary and two buttons.
3. User taps "Approve" → GUI sends `approve` message with `proposal_id`.
4. User taps "Reject" → GUI sends `dismiss` message with `proposal_id`.
5. agntd receives the response and continues the agent turn, feeding the result back to the LLM.

The agent turn is paused during approval — no further `token` or `tool_call` messages until the user responds.

### KS-005: Status Page

Displays real-time system and agent status:

- **Agent card**: connection state (connected/connecting/error), profile name, model name, endpoint.
- **System card**: CPU, RAM, disk, failed units (sourced from `inspect system` on page open and push events).
- **Watchdog card**: last check time, interval, disk threshold, alert count.
- Auto-refresh: watchdog alerts and rebuild events arrive as push `event` messages; system stats refresh on page focus.

### KS-006: Proposals Page

Lists all pending proposals from `/etc/agntos/proposals/*.json`:

- Each proposal card shows: ID, summary, creation timestamp.
- Two action buttons: Apply (sends `approve`), Dismiss (sends `dismiss` with reason "user dismissed").
- Applied proposals show a checkmark and grayed-out buttons.
- New proposals auto-appear via `event: proposal_created` push events.
- Tap a proposal card to expand and see full nix_changes and rollback_guidance.

### KS-007: Activity Page

A scrollable timeline of all audited actions, searchable and filterable.

- **Search bar** at top — sends `audit` request with a query string; matches against summary, prompt, rationale, files changed, and action type.
- **Entry cards** show: ID (truncated), timestamp (relative: "2 min ago"), status icon (✓/✗/⏳), and summary line.
- **Expand a card** to see full details: original prompt, rationale, files changed, rollback hint, and result message.
- **Rollback button** on successful `Apply` and `Rollback` entries — sends a `chat` message to the agent: "rollback audit entry {id}".
- **Color coding**: green tint for success, red for failed, yellow for pending.
- **Auto-append**: new audit entries arrive as push events (`event: audit_entry`), prepended to the list without a full refresh.
- **Initial load**: on page open, request the last 50 entries via `audit` request with `action: "list"`, `limit: 50`.

### KS-008: Push Events

agntd pushes events to all connected clients:

| Event | Trigger | Data |
|---|---|---|
| `watchdog_alert` | Watchdog check trips | `check`, `severity`, `timestamp` |
| `proposal_created` | `agntctl propose` writes a new file | `proposal_id`, `summary` |
| `rebuild_started` | `nixos-rebuild` begins | `proposal_id` |
| `rebuild_complete` | `nixos-rebuild` finishes | `proposal_id`, `success`, `generation` |
| `audit_entry` | New entry written to audit.jsonl | Full `AuditEntry` object |

The GUI uses these to:
- Show a notification badge on the Proposals tab when a new proposal arrives.
- Update the Status page watchdog card when an alert fires.
- Show rebuild progress in the Status page.
- Append new entries to the Activity page in real-time without full refresh.

## Wire Protocol

### Client → Server

```json
{"type": "init", "config_dir": "/etc/agntos"}
{"type": "chat", "prompt": "install nginx"}
{"type": "approve", "proposal_id": "p-abc123"}
{"type": "dismiss", "proposal_id": "p-abc123", "reason": "not needed"}
{"type": "status", "target": "system"}
{"type": "cancel"}
{"type": "audit", "action": "list", "limit": 50}
{"type": "audit", "action": "search", "query": "nginx"}
{"type": "audit", "action": "show", "id": "a-18af76"}
```

### Server → Client

```json
{"type": "session_ready", "profile": "local-35b", "model": "Qwen3.6-35B-A3B-MTP-Q4_K_M", "pending_proposals": ["p-abc123"]}
{"type": "status_response", "target": "system", "data": {"cpu": "8 cores", "ram": "4.2 / 32 GB", "disk": "45% (120 GB free)", "failed_units": 0}}
{"type": "token", "content": "I'll"}
{"type": "token", "content": " install"}
{"type": "token", "content": " nginx"}
{"type": "tool_call", "id": "tc-1", "name": "inspect", "args": {"target": "system"}, "status": "running"}
{"type": "tool_call", "id": "tc-1", "name": "inspect", "args": {"target": "system"}, "status": "done"}
{"type": "tool_result", "id": "tc-1", "name": "inspect", "output": "CPU: 8 cores...", "success": true}
{"type": "approval_request", "proposal_id": "p-abc123", "summary": "Install nginx", "tool_call_id": "tc-2"}
{"type": "turn_complete", "content": "I've proposed installing nginx."}
{"type": "audit_response", "entries": [{"id": "a-18af76", "timestamp": "2025-05-16T14:30:00Z", "action": {"type": "Apply", "proposal_id": "p-abc123"}, "actor": "agent", "summary": "Applied: Install nginx", "result": {"status": "Success", "message": "Rebuild successful"}, "prompt": "install nginx", "files_changed": ["/etc/agntos/generated/packages.nix"]}]}
{"type": "event", "event": "watchdog_alert", "data": {"check": "disk", "severity": "warning", "usage_pct": 96, "timestamp": "2025-05-16T14:30:00Z"}}
{"type": "event", "event": "proposal_created", "data": {"proposal_id": "p-xyz", "summary": "Install htop"}}
{"type": "event", "event": "rebuild_started", "data": {"proposal_id": "p-abc123"}}
{"type": "event", "event": "rebuild_complete", "data": {"proposal_id": "p-abc123", "success": true, "generation": 42}}
{"type": "error", "message": "LLM endpoint unreachable"}
```

### Backward Compatibility

The existing one-shot protocol `{"prompt": "..."}` → `{"response": "..."}` remains functional. When agntd receives a message with a `prompt` field and no `type` field, it falls back to the existing `process_prompt()` path. This preserves `socat` and script compatibility.

## Chat UX Specification

### Text Streaming

- Tokens arrive as `{"type": "token", "content": "..."}` messages.
- The GUI batches tokens on a 30ms timer. Each batch appends to the current message bubble.
- New text fades in over 80ms (QML `PropertyAnimation` on `opacity` from 0 to 1).
- This creates a smooth, readable flow without the jarring character-by-character effect.

### Tool Progress Animation

When a `tool_call` with `status: "running"` arrives:

```
┌─ 🔍 Inspecting system… ────────────┐
│  ░░░ spinner ░░░                      │
└──────────────────────────────────────┘
```

- Compact single-line card with spinner.
- Tool name and short present-tense description.
- Descriptions per tool:

| Tool | Running label |
|---|---|
| `inspect` | "Inspecting {target}…" |
| `propose` | "Creating proposal…" |
| `apply` | "Applying {proposal_id}…" |
| `rollback` | "Rolling back…" |
| `audit` | "Searching audit log…" |
| `memory` | "Updating memory…" |
| `read_file` | "Reading {path}…" |
| `write_file` | "Writing {path}…" |
| `edit_file` | "Editing {path}…" |
| `run_bash` | "Running: {command_truncated}…" |

When `tool_result` arrives, the card transitions:
- Spinner → checkmark (success) or X icon (error).
- Summary line: "Inspected system" or "Created proposal p-abc123: Install nginx".
- Expandable: tap to see full output (truncated to 5 lines, "Show more" for full output).
- Cards default to collapsed after completion.

### Approval Cards

When `approval_request` arrives:

```
┌─ ⚠️  Apply proposal p-abc123? ──────┐
│  Install nginx for web serving       │
│                                      │
│  [✓ Approve]    [✗ Reject]           │
└──────────────────────────────────────┘
```

- Distinct styling: tinted background (Kirigami `warning` color).
- Two buttons, no dismiss-on-outside-click.
- On Approve: sends `approve`, card transitions to "Applying…" spinner, then result.
- On Reject: sends `dismiss`, card shows "Rejected" state.
- The chat is paused — no new `token` or `tool_call` messages until approval is resolved.

### Message Bubble Styling

- User messages: right-aligned, primary color background.
- Agent text: left-aligned, surface color background.
- Tool cards: left-aligned, indented slightly, with a left border accent color per tool type.
- Approval cards: centered, warning-tinted background, with shadow elevation.
- Scrolling auto-follows new content (Qt `ListView` with `positionViewAtEnd()`).

## App Architecture

### Crate: `agntos-settings`

```
crates/agntos-settings/
  Cargo.toml
  src/
    main.rs                 # QML engine setup, cxx-qt bridge registration
    backend/
      mod.rs                # Socket connection, reconnect loop
      protocol.rs           # Serde structs for NDJSON messages
      session.rs            # Session state machine (init → ready → chatting)
    models/
      mod.rs                # cxx-qt bridge module
      chat_model.rs         # QAbstractListModel for chat messages
      proposal_model.rs     # QAbstractListModel for proposals
      status_model.rs       # QObject with status properties
      audit_model.rs        # QAbstractListModel for audit entries
  resources/
    main.qml                # ApplicationWindow + Kirigami drawer
    ChatPage.qml            # Chat panel
    StatusPage.qml           # Agent & system status
    ProposalsPage.qml        # Proposals list
    ActivityPage.qml         # Audit log timeline
    components/
      ToolCallCard.qml       # Collapsible tool result card
      ApprovalCard.qml       # Approve/reject inline card
      MessageBubble.qml      # Chat message bubble
      StatusIndicator.qml    # Connection state dot
  qml.qrc                   # Qt resource file
```

### Rust ↔ QML Bridge (cxx-qt)

The Rust side owns socket communication and protocol state. QML owns rendering.

**Rust → QML (exposed as Q_PROPERTY or Q_SIGNAL):**
- `chatModel` — `ChatModel` (QAbstractListModel): append, update, clear
- `proposalModel` — `ProposalModel` (QAbstractListModel): refresh from `/etc/agntos/proposals/`
- `connected` — `bool` Q_PROPERTY: connection state
- `agentProfile` / `agentModel` — `QString` Q_PROPERTY: model info from `session_ready`
- `statusData` — `StatusModel` (QObject): CPU, RAM, disk, failed units, watchdog state
- `auditModel` — `AuditModel` (QAbstractListModel): audit log entries

**QML → Rust (invokable Q_INVOKABLE methods):**
- `sendChat(prompt: string)` — send a chat message
- `approve(proposalId: string)` — approve a proposal
- `dismiss(proposalId: string, reason: string)` — dismiss a proposal
- `refreshStatus()` — trigger an `inspect system` call
- `refreshProposals()` — re-read proposals directory
- `loadAudit(limit: int)` — request last N audit entries
- `searchAudit(query: string)` — search audit log
- `showAuditEntry(id: string)` — get full detail for one entry
- `rollbackAuditEntry(id: string)` — send rollback chat message

**Signal flow:**
1. Rust reads NDJSON from socket → deserializes → updates model → emits Q_SIGNAL.
2. QML binds to model changes → renders new messages, updates cards.
3. User action in QML → calls Q_INVOKABLE → Rust serializes message → writes to socket.

### Session State Machine

```
         ┌──────────┐
         │ Disconnected │
         └─────┬──────┘
               │ connect()
               ▼
         ┌──────────┐
         │  InitSent  │
         └─────┬──────┘
               │ session_ready
               ▼
         ┌──────────┐
         │   Ready    │◄─────── turn_complete
         └─────┬──────┘
               │ chat()
               ▼
         ┌──────────┐
         │ Chatting   │◄─────── token, tool_call, tool_result
         └─────┬──────┘
               │ approval_request
               ▼
         ┌──────────┐
         │ Awaiting  │─────── approve/dismiss ──► Ready
         │ Approval  │
         └──────────┘
```

On disconnect: transition to `Disconnected`, start reconnect with exponential backoff (1s, 2s, 4s, max 30s).

## NixOS Integration

### agntd Changes

The `--socket` mode in `agntd` gains a persistent session path alongside the existing one-shot path:

- Detect `{"type": "init", ...}` → enter persistent mode.
- Detect `{"prompt": "..."}` → fall back to existing one-shot mode.
- In persistent mode: stream `token` messages for LLM output.
- In persistent mode: send `approval_request` instead of reading stdin for confirmation.
- In persistent mode: push `event` messages for watchdog alerts, proposal creation, rebuild completion.
- The watchdog loop already runs in the background; wire it to broadcast events to all connected persistent clients.

### New NixOS Module: `agntos-settings.nix`

```nix
{ config, pkgs, lib, ... }:
{
  options.agntos.settings = {
    enable = lib.mkEnableOption "AgntOS Settings GUI";
  };
  config = lib.mkIf config.agntos.settings.enable {
    environment.systemPackages = [ pkgs.agntos-settings ];
    # .desktop file for app launcher
    # Autostart optional (agntd already starts via agent.nix)
  };
}
```

### Build Integration

- `agntos-settings` added to the Cargo workspace.
- QML resources compiled via `qt5.qt Mature` or Qt6's `qt_add_qml_module`.
- cxx-qt generates the bridge code at build time.
- Nix flake adds a `packages.agntos-settings` output.

## Deferred

These items are explicitly out of scope for Phase 3 v1:

- **Model routing page** — `agntctl model` CLI suffices; GUI page deferred.
- **Memory viewer/editor** — `agntctl memory show/add/consolidate` CLI suffices.
- **Watchdog configuration GUI** — `/etc/agntos/watchdog.toml` editing via `agntctl propose set-home-option` or `edit_file`.
- **Home Manager GUI** — same as watchdog: use propose/edit_file.
- **D-Bus interface** — may add later for Plasma integration; not needed for v1.
- **Multiple simultaneous clients** — persistent mode supports one GUI; one-shot mode for scripts remains.

## Exit Criteria

- User can chat with the agent through the GUI with streamed, fade-in responses.
- Tool calls render as inline cards with spinners during execution and expandable results.
- Approval-gated actions (apply, rollback) show Approve/Reject buttons in the chat.
- Status page shows agent connection, system info, and watchdog state.
- Proposals page lists pending proposals with Apply/Dismiss actions.
- Activity page shows audit timeline with search, expand for details, and rollback actions.
- Push events update proposals, watchdog status, and activity in real-time.
- Existing one-shot socket protocol still works for `socat` and scripts.