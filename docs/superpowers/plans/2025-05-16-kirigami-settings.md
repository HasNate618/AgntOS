# Kirigami Settings — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build a standalone Kirigami GUI app (`agntos-settings`) that provides chat-driven interaction with the AgntOS agent, plus dashboard pages for status, proposals, and audit history.

**Architecture:** The app uses QML + Rust via cxx-qt. A persistent NDJSON-over-Unix-socket connection to `agntd` provides streaming chat, tool call progress, approval flow, and push events. The Rust backend owns the protocol, state machine, and data models. QML renders the UI with native Kirigami components.

**Tech Stack:** Rust (cxx-qt bridge), QML/Kirigami, Qt6, tokio, serde_json, agnt_common crate

**Spec:** `.specs/features/kirigami-settings/spec.md` and `design.md`

---

## File Structure

Files are organized by responsibility. Each file has one clear purpose.

### New files (agntos-settings crate)

```
crates/agntos-settings/
  Cargo.toml                    # cxx-qt, serde, tokio deps
  src/
    main.rs                     # QML engine, cxx-qt bridge registration
    backend/
      mod.rs                    # Socket connection, reconnect loop
      protocol.rs               # Serde message types for NDJSON
      session.rs                # State machine (Disconnected→Ready→Chatting→AwaitingApproval)
    models/
      mod.rs                    # cxx-qt bridge module
      chat_model.rs             # QAbstractListModel: chat messages, tool cards, approvals
      proposal_model.rs         # QAbstractListModel: pending proposals
      status_model.rs           # QObject: agent/system/watchdog status properties
      audit_model.rs            # QAbstractListModel: audit log entries
  resources/
    main.qml                    # ApplicationWindow + Kirigami global drawer
    ChatPage.qml                # Chat panel with inline tool cards
    StatusPage.qml              # Agent & system status dashboard
    ProposalsPage.qml            # Proposals list with apply/dismiss
    ActivityPage.qml            # Audit log timeline with search
    components/
      ToolCallCard.qml          # Collapsible tool result card (spinner→result)
      ApprovalCard.qml          # Approve/reject inline card
      MessageBubble.qml         # Chat message bubble (user/assistant)
      StatusIndicator.qml       # Connection state dot
      AuditEntryCard.qml        # Expandable audit entry card
  qml.qrc                       # Qt resource catalog
```

### Modified files (agntd)

```
crates/agntd/src/
  main.rs                       # Add persistent session handler alongside existing one-shot
  agent.rs                      # Add approval gate, audit event broadcasting
  session.rs                    # No changes (already complete)
  llm.rs                        # No changes (protocol is same LLM calls, just different delivery)
  watchdog.rs                   # Add broadcast channel for watchdog alerts
```

### Modified files (agnt_common)

```
crates/agnt-common/src/
  lib.rs                        # Add pub mod wire; (new wire protocol module)
  wire.rs                       # Shared NDJSON message types (used by both agntd and agntos-settings)
```

### Modified files (NixOS)

```
modules/agntos/
  agntos-settings.nix           # New: NixOS module for the GUI package
  base.nix                      # Add agntos-settings package when agntos.settings.enable = true
```

### Modified files (workspace)

```
Cargo.toml                      # Add agntos-settings to workspace members
flake.nix                       # Add agntos-settings package output
```

---

## Task Decomposition

This plan is ordered by dependency: shared types first, then backend protocol, then agntd changes, then GUI models, then QML pages.

---

### Task 1: Shared Wire Protocol Types

**Files:**
- Create: `crates/agnt-common/src/wire.rs`
- Modify: `crates/agnt-common/src/lib.rs`
- Modify: `crates/agnt-common/Cargo.toml`

Shared NDJSON message types used by both `agntd` and `agntos-settings`.

- [ ] **Step 1: Add serde dependency to agnt_common**

In `crates/agnt-common/Cargo.toml`, add:

```toml
[dependencies]
serde = { version = "1", features = ["derive"] }
serde_json = "1"
chrono = { version = "0.4", features = ["serde"] }
```

- [ ] **Step 2: Create wire.rs with all message types**

Create `crates/agnt-common/src/wire.rs`:

```rust
use serde::{Deserialize, Serialize};

// ── Client → Server ──────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ClientMessage {
    #[serde(rename = "init")]
    Init {
        #[serde(default)]
        config_dir: Option<String>,
    },
    #[serde(rename = "chat")]
    Chat { prompt: String },
    #[serde(rename = "approve")]
    Approve { proposal_id: String },
    #[serde(rename = "dismiss")]
    Dismiss {
        proposal_id: String,
        #[serde(default)]
        reason: Option<String>,
    },
    #[serde(rename = "status")]
    Status {
        #[serde(default = "default_system")]
        target: String,
    },
    #[serde(rename = "audit")]
    Audit {
        action: AuditRequestAction,
        #[serde(default)]
        query: Option<String>,
        #[serde(default)]
        id: Option<String>,
        #[serde(default = "default_limit")]
        limit: u32,
    },
    #[serde(rename = "cancel")]
    Cancel {},
}

fn default_system() -> String {
    "system".to_string()
}
fn default_limit() -> u32 {
    50
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "action")]
pub enum AuditRequestAction {
    #[serde(rename = "list")]
    List,
    #[serde(rename = "search")]
    Search,
    #[serde(rename = "show")]
    Show,
}

// ── Server → Client ──────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ServerMessage {
    #[serde(rename = "session_ready")]
    SessionReady {
        profile: String,
        model: String,
        #[serde(default)]
        pending_proposals: Vec<String>,
    },
    #[serde(rename = "status_response")]
    StatusResponse {
        target: String,
        data: serde_json::Value,
    },
    #[serde(rename = "token")]
    Token { content: String },
    #[serde(rename = "tool_call")]
    ToolCall {
        id: String,
        name: String,
        args: serde_json::Value,
        status: ToolCallStatus,
    },
    #[serde(rename = "tool_result")]
    ToolResult {
        id: String,
        name: String,
        output: String,
        success: bool,
    },
    #[serde(rename = "approval_request")]
    ApprovalRequest {
        proposal_id: String,
        summary: String,
        tool_call_id: String,
    },
    #[serde(rename = "turn_complete")]
    TurnComplete { content: String },
    #[serde(rename = "audit_response")]
    AuditResponse {
        entries: Vec<serde_json::Value>,
    },
    #[serde(rename = "event")]
    Event {
        event: String,
        data: serde_json::Value,
    },
    #[serde(rename = "error")]
    Error { message: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "status")]
pub enum ToolCallStatus {
    #[serde(rename = "running")]
    Running,
    #[serde(rename = "done")]
    Done,
}

// ── Backward compat ──────────────────────────────────────────────────────

/// Detect whether a raw JSON string is a legacy one-shot `{"prompt": "..."}` message
/// or a new typed `{"type": "..."}`  message.
pub fn is_legacy_prompt(raw: &str) -> bool {
    // Legacy messages have "prompt" key but no "type" key
    raw.contains("\"prompt\"") && !raw.contains("\"type\"")
}
```

- [ ] **Step 3: Register wire module in lib.rs**

In `crates/agnt-common/src/lib.rs`, add:

```rust
pub mod wire;
```

- [ ] **Step 4: Run existing tests**

Run: `cargo test -p agnt-common`
Expected: 6 tests pass (existing tests unchanged)

- [ ] **Step 5: Add wire type tests**

Add to `crates/agnt-common/src/wire.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn client_message_init_serialization() {
        let msg = ClientMessage::Init { config_dir: None };
        let json = serde_json::to_string(&msg).unwrap();
        assert!(json.contains("\"type\":\"init\""));
    }

    #[test]
    fn client_message_chat_serialization() {
        let msg = ClientMessage::Chat { prompt: "install nginx".to_string() };
        let json = serde_json::to_string(&msg).unwrap();
        assert!(json.contains("\"type\":\"chat\""));
        assert!(json.contains("install nginx"));
    }

    #[test]
    fn server_message_token_deserialization() {
        let json = r#"{"type":"token","content":"hello"}"#;
        let msg: ServerMessage = serde_json::from_str(json).unwrap();
        match msg {
            ServerMessage::Token { content } => assert_eq!(content, "hello"),
            _ => panic!("expected Token"),
        }
    }

    #[test]
    fn is_legacy_prompt_detection() {
        assert!(is_legacy_prompt(r#"{"prompt":"hello"}"#));
        assert!(!is_legacy_prompt(r#"{"type":"init"}"#));
        assert!(!is_legacy_prompt(r#"{"type":"chat","prompt":"hello"}"#));
    }

    #[test]
    fn roundtrip_client_messages() {
        let messages = vec![
            ClientMessage::Approve { proposal_id: "p-abc".to_string() },
            ClientMessage::Dismiss { proposal_id: "p-abc".to_string(), reason: Some("not needed".to_string()) },
            ClientMessage::Status { target: "system".to_string() },
            ClientMessage::Cancel {},
        ];
        for msg in &messages {
            let json = serde_json::to_string(msg).unwrap();
            let back: ClientMessage = serde_json::from_str(&json).unwrap();
            let json2 = serde_json::to_string(&back).unwrap();
            assert_eq!(json, json2);
        }
    }

    #[test]
    fn roundtrip_server_messages() {
        let messages = vec![
            ServerMessage::SessionReady { profile: "local".to_string(), model: "qwen".to_string(), pending_proposals: vec![] },
            ServerMessage::ToolCall { id: "tc-1".to_string(), name: "inspect".to_string(), args: serde_json::json!({}), status: ToolCallStatus::Running },
            ServerMessage::Error { message: "test".to_string() },
        ];
        for msg in &messages {
            let json = serde_json::to_string(msg).unwrap();
            let back: ServerMessage = serde_json::from_str(&json).unwrap();
            let json2 = serde_json::to_string(&back).unwrap();
            assert_eq!(json, json2);
        }
    }
}
```

- [ ] **Step 6: Run tests**

Run: `cargo test -p agnt-common`
Expected: All tests pass (6 existing + 6 new = 12)

- [ ] **Step 7: Commit**

```bash
git add crates/agnt-common/src/wire.rs crates/agnt-common/src/lib.rs crates/agnt-common/Cargo.toml
git commit -m "feat: shared wire protocol types for GUI socket communication"
```

---

### Task 2: agntd Persistent Session Handler

**Files:**
- Modify: `crates/agntd/src/main.rs`
- Modify: `crates/agntd/src/agent.rs`
- Modify: `crates/agntd/Cargo.toml`

Add persistent session mode to agntd's socket handler. When a client sends `{"type":"init",...}`, agntd keeps the connection open and streams responses. Legacy `{"prompt":"..."}` still works as before.

- [ ] **Step 1: Add tokio broadcast dependency**

In `crates/agntd/Cargo.toml`, add to `[dependencies]`:

```toml
tokio = { version = "1", features = ["sync"] }
```

- [ ] **Step 2: Add approval gate type to agent.rs**

Add near the top of `crates/agntd/src/agent.rs`:

```rust
use std::sync::{Arc, Mutex};

pub struct ApprovalGate {
    pub proposal_id: String,
    pub tool_call_id: String,
    pub summary: String,
    pub resolved: bool,
    pub approved: bool,
}

pub type SharedApprovalGate = Arc<Mutex<Option<ApprovalGate>>>;
```

- [ ] **Step 3: Add broadcast channel type to watchdog.rs**

In `crates/agntd/src/watchdog.rs`, add:

```rust
use tokio::sync::broadcast;
use agnt_common::wire::ServerMessage;

pub type EventSender = broadcast::Sender<ServerMessage>;

pub fn create_event_channel() -> EventSender {
    let (tx, _) = broadcast::channel(32);
    tx
}
```

- [ ] **Step 4: Add persistent session handler in main.rs**

In `crates/agntd/src/main.rs`, add a `run_persistent_session` function that handles the new protocol. This function:

1. Reads the first message from the socket.
2. If it's a legacy `{"prompt":"..."}` message, falls back to `process_prompt()`.
3. If it's a `{"type":"init"}` message, enters persistent mode:
   - Sends `session_ready`.
   - Reads messages in a loop, dispatching by type (`chat`, `approve`, `dismiss`, `status`, `audit`, `cancel`).
   - For `chat`: calls the agent turn loop but streams `token`, `tool_call`, `tool_result`, `approval_request`, `turn_complete` messages.
   - For `status`: calls `inspect` and sends `status_response`.
   - For `audit`: calls `agntctl audit` and sends `audit_response`.
   - For `approve`/`dismiss`: resolves the approval gate and continues the agent turn.
   - Subscribes to the broadcast channel for push events.

The implementation should be a new function `handle_persistent_connection` in `main.rs` that takes the `DaemonBootstrap`, the stream, and the event sender.

Key design: the approval gate is a shared `Arc<Mutex<Option<ApprovalGate>>>` that the persistent handler sets when the agent requests approval, and resolves when the client sends `approve`/`dismiss`. The agent loop checks the gate and yields control when pending.

- [ ] **Step 5: Send push events from watchdog**

In `crates/agntd/src/watchdog.rs`, after the watchdog triage logic, send `ServerMessage::Event` through the broadcast channel when a check trips or resolves.

- [ ] **Step 6: Wire broadcast channel into main.rs**

In `main.rs`, create the broadcast channel before the listener loop and pass it to both the watchdog start and the persistent session handler.

- [ ] **Step 7: Run existing tests**

Run: `cargo test -p agntd`
Expected: 22 tests pass (existing tests unchanged — this only adds the persistent path)

- [ ] **Step 8: Commit**

```bash
git add crates/agntd/src/main.rs crates/agntd/src/agent.rs crates/agntd/src/watchdog.rs crates/agntd/Cargo.toml
git commit -m "feat: agntd persistent session handler with streaming and approval gates"
```

---

### Task 3: agntos-settings Crate Scaffold

**Files:**
- Create: `crates/agntos-settings/Cargo.toml`
- Create: `crates/agntos-settings/src/main.rs`
- Create: `crates/agntos-settings/src/backend/mod.rs`
- Create: `crates/agntos-settings/src/backend/protocol.rs`
- Create: `crates/agntos-settings/src/backend/session.rs`
- Create: `crates/agntos-settings/src/models/mod.rs`
- Modify: `Cargo.toml` (workspace)

Set up the crate structure, Cargo.toml with cxx-qt and dependencies, and stub out the modules.

- [ ] **Step 1: Add agntos-settings to workspace**

In root `Cargo.toml`, add to `[workspace].members`:

```toml
members = [
    "crates/agnt-common",
    "crates/agntctl",
    "crates/agntd",
    "crates/agntos-settings",
]
```

- [ ] **Step 2: Create Cargo.toml**

Create `crates/agntos-settings/Cargo.toml`:

```toml
[package]
name = "agntos-settings"
version = "0.1.0"
edition = "2021"

[dependencies]
agnt-common = { path = "../agnt-common" }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
tokio = { version = "1", features = ["sync", "rt-multi-thread", "net", "io-util"] }
cxx-qt = "0.6"
cxx-qt-build = "0.6"

[build-dependencies]
cxx-qt-build = "0.6"
```

- [ ] **Step 3: Create stub source files**

Create `crates/agntos-settings/src/main.rs`:

```rust
fn main() {
    println!("agntos-settings: Kirigami GUI for AgntOS");
    println!("TODO: initialize QML engine and cxx-qt bridge");
}
```

Create `crates/agntos-settings/src/backend/mod.rs`:

```rust
pub mod protocol;
pub mod session;
```

Create `crates/agntos-settings/src/backend/protocol.rs`:

```rust
use agnt_common::wire::{ClientMessage, ServerMessage};

pub fn serialize(msg: &ClientMessage) -> String {
    serde_json::to_string(msg).unwrap()
}

pub fn deserialize(raw: &str) -> Result<ServerMessage, String> {
    serde_json::from_str(raw).map_err(|e| format!("Protocol error: {}", e))
}
```

Create `crates/agntos-settings/src/backend/session.rs`:

```rust
pub enum SessionState {
    Disconnected,
    InitSent,
    Ready,
    Chatting,
    AwaitingApproval,
}

pub struct Session {
    pub state: SessionState,
    pub profile: String,
    pub model: String,
    pub pending_proposals: Vec<String>,
}

impl Session {
    pub fn new() -> Self {
        Self {
            state: SessionState::Disconnected,
            profile: String::new(),
            model: String::new(),
            pending_proposals: Vec::new(),
        }
    }
}
```

Create `crates/agntos-settings/src/models/mod.rs`:

```rust
// Models will be cxx-qt bridge types.
// Stubs for now — actual QAbstractListModel implementations come later.
```

- [ ] **Step 4: Verify it compiles**

Run: `cargo check -p agntos-settings`
Expected: Compiles with only warnings about unused code (stubs)

- [ ] **Step 5: Commit**

```bash
git add Cargo.toml crates/agntos-settings/
git commit -m "feat: agntos-settings crate scaffold with protocol and session stubs"
```

---

### Task 4: Socket Connection and Reconnect Loop

**Files:**
- Modify: `crates/agntos-settings/src/backend/mod.rs`
- Modify: `crates/agntos-settings/src/backend/session.rs`

Implement the Unix socket connection, NDJSON line protocol read/write, reconnect with exponential backoff, and message dispatch.

- [ ] **Step 1: Implement socket connection in mod.rs**

Add a `Connection` struct that owns a UnixStream, reads NDJSON lines, writes serialized `ClientMessage`s, and handles reconnection. Methods:

- `connect(path: &str) -> Result<Self, String>`: Connect to socket, create buffered reader/writer.
- `send(&mut self, msg: &ClientMessage) -> Result<(), String>`: Serialize and write newline-terminated.
- `recv(&mut self) -> Result<ServerMessage, String>`: Read a line, deserialize.
- `reconnect(&mut self, path: &str) -> Result<(), String>`: Close, reconnect, re-send `init`.

- [ ] **Step 2: Implement session state machine**

Add state transitions:

- `connect()` → send `init` → transition to `InitSent`
- Receive `session_ready` → transition to `Ready`, store profile/model/proposals
- `send_chat()` → transition to `Chatting`
- Receive `approval_request` → transition to `AwaitingApproval`
- `approve()`/`dismiss()` → transition back to `Chatting` (agent turn continues)
- Receive `turn_complete` → transition to `Ready`
- Socket error → transition to `Disconnected`, start backoff

- [ ] **Step 3: Test with local socket connection**

Write a test that starts a mock agntd on a temp socket, sends `init`, receives `session_ready`, sends `chat`, and verifies state transitions. This test can use the existing agntd binary in `--socket` mode with a test config directory.

- [ ] **Step 4: Commit**

```bash
git add crates/agntos-settings/src/backend/
git commit -m "feat: socket connection, NDJSON protocol, and session state machine"
```

---

### Task 5: Data Models (cxx-qt Bridge)

**Files:**
- Create: `crates/agntos-settings/src/models/chat_model.rs`
- Create: `crates/agntos-settings/src/models/proposal_model.rs`
- Create: `crates/agntos-settings/src/models/status_model.rs`
- Create: `crates/agntos-settings/src/models/audit_model.rs`
- Modify: `crates/agntos-settings/src/models/mod.rs`

Implement the four QAbstractListModel/QObject types with cxx-qt bridges. Each model exposes Q_PROPERTY and Q_INVOKABLE methods for QML binding.

- [ ] **Step 1: Implement ChatModel**

`QAbstractListModel` with roles: `typeRole` (user/assistant/tool_call/tool_result/approval), `contentRole`, `toolNameRole`, `toolArgsRole`, `toolOutputRole`, `toolSuccessRole`, `proposalIdRole`, `proposalSummaryRole`, `timestampRole`.

Methods: `appendUserMessage(prompt)`, `appendToken(content)` (batched), `addToolCall(id, name, args)`, `resolveToolCall(id, output, success)`, `addApprovalRequest(proposal_id, summary, tool_call_id)`, `resolveApproval(proposal_id, approved)`, `clear()`.

- [ ] **Step 2: Implement ProposalModel**

`QAbstractListModel` with roles: `proposalIdRole`, `summaryRole`, `createdAtRole`, `statusRole`.

Methods: `refresh()` (reads `/etc/agntos/proposals/*.json`), `apply(proposal_id)` (sends `approve` message), `dismiss(proposal_id, reason)` (sends `dismiss` message).

- [ ] **Step 3: Implement StatusModel**

`QObject` with Q_PROPERTY: `connected`, `profileName`, `modelName`, `endpoint`, `cpuInfo`, `ramUsed`, `diskUsed`, `failedUnits`, `watchdogInterval`, `watchdogDiskThreshold`, `watchdogAlertCount`, `lastCheckTime`.

Methods: `refresh()` (sends `status` request).

- [ ] **Step 4: Implement AuditModel**

`QAbstractListModel` with roles: `auditIdRole`, `timestampRole`, `actionTypeRole`, `summaryRole`, `statusRole` (success/failed/pending), `actorRole`, `promptRole`, `rationaleRole`, `filesChangedRole`, `rollbackHintRole`, `resultMessageRole`.

Methods: `load(limit)` (sends audit list request), `search(query)`, `show(id)`, `rollback(id)` (sends chat message).

- [ ] **Step 5: Wire models into mod.rs**

Register all models with cxx-qt bridge.

- [ ] **Step 6: Compile and verify**

Run: `cargo check -p agntos-settings`

- [ ] **Step 7: Commit**

```bash
git add crates/agntos-settings/src/models/
git commit -m "feat: data models — ChatModel, ProposalModel, StatusModel, AuditModel"
```

---

### Task 6: QML Pages — Chat

**Files:**
- Create: `crates/agntos-settings/resources/main.qml`
- Create: `crates/agntos-settings/resources/ChatPage.qml`
- Create: `crates/agntos-settings/resources/components/MessageBubble.qml`
- Create: `crates/agntos-settings/resources/components/ToolCallCard.qml`
- Create: `crates/agntos-settings/resources/components/ApprovalCard.qml`
- Create: `crates/agntos-settings/resources/components/StatusIndicator.qml`
- Create: `crates/agntos-settings/resources/qml.qrc`

Implement the main application window with Kirigami drawer and the Chat page with all its components.

- [ ] **Step 1: Create main.qml**

Kirigami `ApplicationWindow` with global drawer containing four pages: Chat, Status, Proposals, Activity. The drawer footer shows agent connection status (green dot = connected, red = disconnected, yellow = connecting).

- [ ] **Step 2: Create ChatPage.qml**

`Kirigami.Page` with:
- `ListView` bound to `ChatModel` for the message list.
- Input field at bottom with send button.
- `positionViewAtEnd()` on new messages.
- Auto-scroll follows new content.

- [ ] **Step 3: Create MessageBubble.qml**

Delegate for chat messages:
- User messages: right-aligned, primary color.
- Assistant text: left-aligned, surface color.
- Fade-in animation on new text (80ms, `Easing.OutQuad`).

- [ ] **Step 4: Create ToolCallCard.qml**

Delegate for tool calls:
- Running state: `BusyIndicator` + "Inspecting system…" label.
- Done state: checkmark/X icon + collapsible output (5 lines max, "Show more" button).
- Left border accent color by tool type (blue for inspect, green for propose, orange for apply, etc.)

- [ ] **Step 5: Create ApprovalCard.qml**

Delegate for approval requests:
- Warning-tinted background.
- Summary text.
- Two buttons: Approve (green) and Reject (red).
- On click: calls `approve(proposalId)` or `dismiss(proposalId, "user rejected")`.

- [ ] **Step 6: Create StatusIndicator.qml**

Small circular indicator: green = connected, red = error, yellow = connecting. Bound to `connected` Q_PROPERTY.

- [ ] **Step 7: Create qml.qrc**

Qt resource catalog listing all QML and asset files.

- [ ] **Step 8: Verify QML loads without runtime errors**

Build and run the app. It should show an empty window with the drawer. No crashes.

- [ ] **Step 9: Commit**

```bash
git add crates/agntos-settings/resources/
git commit -m "feat: QML pages — Chat with inline tool cards and approval flow"
```

---

### Task 7: QML Pages — Status, Proposals, Activity

**Files:**
- Create: `crates/agntos-settings/resources/StatusPage.qml`
- Create: `crates/agntos-settings/resources/ProposalsPage.qml`
- Create: `crates/agntos-settings/resources/ActivityPage.qml`
- Create: `crates/agntos-settings/resources/components/AuditEntryCard.qml`

Implement the three dashboard pages.

- [ ] **Step 1: Create StatusPage.qml**

`Kirigami.Page` with three `Kirigami.FormSection` cards:
- Agent card: connection state, profile name, model name, endpoint.
- System card: CPU, RAM, disk, failed units (from `StatusModel`).
- Watchdog card: last check time, interval, disk threshold, alert count.
- Refresh button at bottom.

- [ ] **Step 2: Create ProposalsPage.qml**

`Kirigami.Page` with `ListView` bound to `ProposalModel`:
- Each delegate is a `Kirigami.AbstractCard` showing ID, summary, creation time.
- Two `Kirigami.Action` buttons: Apply (green) and Dismiss (red).
- Applied proposals show a checkmark and grayed-out buttons.
- Pull-to-refresh calls `proposalModel.refresh()`.

- [ ] **Step 3: Create ActivityPage.qml**

`Kirigami.Page` with:
- Search bar at top bound to `AuditModel.search()`.
- `ListView` of audit entries bound to `AuditModel`.
- Pull-to-refresh calls `auditModel.load(50)`.

- [ ] **Step 4: Create AuditEntryCard.qml**

Delegate for audit entries:
- Status icon: ✓ (green), ✗ (red), ⏳ (yellow).
- Summary line + relative timestamp.
- Expand to show: prompt, rationale, files changed, rollback hint.
- Rollback button on Apply entries.

- [ ] **Step 5: Verify all pages render**

Build and run. Navigate between all four pages. Each page should show empty state (or last cached data).

- [ ] **Step 6: Commit**

```bash
git add crates/agntos-settings/resources/
git commit -m "feat: QML pages — Status dashboard, Proposals list, Activity timeline"
```

---

### Task 8: End-to-End Integration Test

**Files:**
- Create: `crates/agntos-settings/tests/integration.rs`
- Modify: `crates/agntos-settings/Cargo.toml` (add dev-dependencies)

Integration test that starts agntd in `--socket` mode, connects the GUI backend, and exercises the full protocol.

- [ ] **Step 1: Add test dependency**

In `crates/agntos-settings/Cargo.toml`, add:

```toml
[dev-dependencies]
tempfile = "3"
```

- [ ] **Step 2: Write integration test**

Test scenario:
1. Start agntd with `--socket` pointing to a temp path (using the test config directory).
2. Connect via the `Connection` struct.
3. Send `init`, receive `session_ready`.
4. Send `chat "what proposals are pending?"`, receive streaming tokens followed by `turn_complete`.
5. Verify `ChatModel` has entries after the turn.
6. Send `status system`, receive `status_response`.
7. Send `audit list`, receive `audit_response`.
8. Disconnect and verify reconnect logic.

Note: This test requires a running LLM endpoint. For CI, mark it `#[ignore]` and add a `--ignored` flag for manual runs.

- [ ] **Step 3: Run cargo test**

Run: `cargo test -p agntos-settings`
Expected: Unit tests pass. Integration test compiles (may be `#[ignore]`d without LLM).

- [ ] **Step 4: Commit**

```bash
git add crates/agntos-settings/tests/
git commit -m "test: integration test for persistent socket protocol"
```

---

### Task 9: Build Integration and NixOS Module

**Files:**
- Modify: `flake.nix`
- Create: `modules/agntos/agntos-settings.nix`
- Modify: `modules/agntos/base.nix`

Package `agntos-settings` in the Nix flake and create a NixOS module that makes the GUI available when enabled.

- [ ] **Step 1: Add agntos-settings package to flake.nix**

Add a `packages.agntos-settings` output using crane or naersk (matching the existing pattern for `agntctl` and `agntd`). The package needs Qt6 and Kirigami as build inputs.

- [ ] **Step 2: Create agntos-settings.nix module**

```nix
{ config, pkgs, lib, ... }:
{
  options.agntos.settings = {
    enable = lib.mkEnableOption "AgntOS Settings GUI";
  };
  config = lib.mkIf config.agntos.settings.enable {
    environment.systemPackages = [ config.agntos.settings.package ];
    # .desktop file included in the package via Qt6 wrapping
  };
}
```

- [ ] **Step 3: Add module to base.nix imports**

In `modules/agntos/base.nix`, add `./agntos-settings.nix` to the imports list.

- [ ] **Step 4: Verify nix build**

Run: `nix build --impure .#packages.x86_64-linux.agntos-settings`
Expected: Package builds (may need Qt6/kirigami inputs in flake.nix first)

- [ ] **Step 5: Commit**

```bash
git add flake.nix modules/agntos/agntos-settings.nix modules/agntos/base.nix
git commit -m "feat: NixOS module and flake package for agntos-settings"
```

---

### Task 10: Documentation and Spec Update

**Files:**
- Modify: `AGENTS.md`
- Modify: `.specs/project/ROADMAP.md`
- Modify: `.specs/project/STATE.md`
- Modify: `.specs/features/kirigami-settings/tasks.md`

Update project docs to reflect the completed Phase 3 v1 implementation.

- [ ] **Step 1: Update ROADMAP.md**

Mark Phase 3 checklist items:
- [x] Chat interface with streaming and tool cards
- [x] Status dashboard
- [x] Proposals manager
- [x] Activity/audit log viewer
- [ ] Model routing page (deferred)
- [ ] Permissions and skills management (deferred)

- [ ] **Step 2: Update STATE.md**

Add a "Phase 3 v1" section documenting:
- Wire protocol evolution (backward compat with one-shot)
- cxx-qt bridge architecture
- Persistent session state machine
- Approval gate pattern

- [ ] **Step 3: Create tasks.md**

Create `.specs/features/kirigami-settings/tasks.md` tracking all tasks T1–T10 with verification criteria from this plan.

- [ ] **Step 4: Update AGENTS.md**

Update the test count and project layout to include `agntos-settings`.

- [ ] **Step 5: Commit**

```bash
git add AGENTS.md .specs/ modules/
git commit -m "docs: update project docs for Phase 3 v1 Kirigami Settings"
```

---

## Verification

After all tasks are complete:

1. `cargo test` — all tests pass (agnt-common, agntctl, agntd, agntos-settings)
2. `cargo build --release` — clean build with zero warnings
3. Start agntd in socket mode, connect the GUI, send `init` → receive `session_ready`
4. Chat with the agent, verify streaming tokens, tool cards, approval flow
5. Verify status page updates, proposals page loads, activity page shows audit log
6. Test backward compat: `echo '{"prompt":"inspect system"}' | socat - UNIX-CONNECT:/run/agntd/agent.sock` still works
7. `nix build --impure .#packages.x86_64-linux.agntos-settings` builds the Nix package