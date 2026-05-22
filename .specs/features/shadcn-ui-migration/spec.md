# shadcn/ui Frontend Migration Specification

## Problem Statement

The Tauri control centre frontend is built with Svelte 5 and custom CSS using ad-hoc CSS variables that don't match the AgntOS design system (`docs/design.md`). The visual language is inconsistent — violet accent instead of the brand's warm orange, no Syne/Plus Jakarta Sans typography, and no shadcn/ui component patterns. We need a production-grade UI framework that implements the AgntOS brand accurately, provides accessible components out of the box, and is maintainable.

## Goals

- [x] Replace Svelte 5 with React + TypeScript + Vite
- [x] Install and configure shadcn/ui with AgntOS design tokens
- [x] Implement all 4 pages (Chat, Status, Proposals, Activity) as shadcn/ui components
- [x] Wire Tauri IPC (`invoke` + `listen`) identically to current implementation
- [x] Preserve all existing functionality — no regressions

## Out of Scope

| Feature | Reason |
| ------- | ------ |
| Model routing page | Deferred elsewhere; `agntctl model` CLI suffices |
| Memory viewer/editor | Deferred; `agntctl memory` CLI suffices |
| Settings page | Not in current Svelte app — would be new scope |
| Dark/light mode toggle | Only dark mode is currently supported |
| Tauri Rust backend changes | Backend stays unchanged — only frontend framework swap |

---

## User Stories

### P1: Chat Page ⭐ MVP

**User Story**: As a user, I want to chat with the agent through a polished chat interface so I can manage my system conversationally.

**Why P1**: The chat page is the primary user-facing interface. Without it, the control centre has no value.

**Acceptance Criteria**:

1. WHEN user types a message and presses Send THEN `invoke("send_prompt")` SHALL be called
2. WHEN `agent:start` event fires THEN a thinking indicator SHALL appear
3. WHEN `agent:message-update` fires with `text_delta` THEN streaming text SHALL render in real-time via markdown
4. WHEN `agent:tool-start` fires THEN a tool call card SHALL render with name and running spinner
5. WHEN `agent:tool-end` fires THEN the tool card SHALL update with result (collapsible)
6. WHEN `agent:approval-request` fires THEN an approval card SHALL appear with Approve/Dismiss buttons
7. WHEN user clicks Approve THEN `invoke("send_extension_ui_response")` SHALL be called with `confirmed: true`
8. WHEN `agent:error` fires THEN an error message SHALL render
9. WHEN messages overflow THEN the message list SHALL auto-scroll to bottom
10. WHEN Enter is pressed (without Shift) THEN Send SHALL be triggered
11. WHEN Shift+Enter is pressed THEN a newline SHALL be inserted

**Independent Test**: Send a prompt, see streaming response, tool calls, and approval card.

---

### P1: Status Page

**User Story**: As a user, I want to see agent connection state, system info, and watchdog health in one place.

**Why P1**: Provides critical system awareness.

**Acceptance Criteria**:

1. WHEN the page loads THEN `invoke("get_connection_status")` SHALL populate agent status
2. WHEN `agent:connected` event fires THEN status SHALL show "Connected" with green dot
3. WHEN `agent:disconnected` event fires THEN status SHALL show "Disconnected" with red dot
4. WHEN `invoke("get_system_info")` resolves THEN CPU, RAM, Disk, failed units SHALL display
5. WHEN watchdog data is available THEN alert count and last check time SHALL display

**Independent Test**: Navigate to Status page, see connection state and system info.

---

### P1: Proposals Page

**User Story**: As a user, I want to view pending Nix proposals and apply or dismiss them.

**Why P1**: Critical for the propose → apply → rollback workflow.

**Acceptance Criteria**:

1. WHEN the page loads THEN `invoke("list_proposals")` SHALL be called
2. WHEN proposals exist THEN each SHALL show as a card with ID, summary, status badge
3. WHEN "Apply" is clicked THEN `invoke("apply_proposal")` SHALL be called
4. WHEN "Dismiss" is clicked THEN the card SHALL be removed from the list
5. WHEN `invoke("list_audit_entries")` returns applied proposals THEN they SHALL show with "Revert" button
6. WHEN "Revert" is clicked THEN `invoke("rollback_to")` SHALL be called

**Independent Test**: Navigate to Proposals, see pending mutations, apply one.

---

### P1: Activity Page

**User Story**: As a user, I want to browse the audit log with search and rollback actions.

**Why P1**: Essential for auditing system changes.

**Acceptance Criteria**:

1. WHEN the page loads THEN `invoke("list_audit_entries", { limit: 50 })` SHALL be called
2. WHEN entries exist THEN each SHALL show action, description, timestamp, audit ID
3. WHEN the search input changes THEN entries SHALL filter by text match
4. WHEN an "apply" entry has a Revert button and it's clicked THEN `invoke("rollback_to")` SHALL be called
5. WHEN the refresh button is clicked THEN entries SHALL reload

**Independent Test**: Navigate to Activity, see audit log, search, revert an entry.

---

### P2: Sidebar Navigation

**User Story**: As a user, I want to switch between pages via a sidebar.

**Why P2**: Core navigation pattern; P2 because pages are otherwise navigable via tabs.

**Acceptance Criteria**:

1. WHEN a sidebar icon is clicked THEN the corresponding page SHALL display
2. WHEN a page is active THEN its sidebar icon SHALL show active styling
3. WHEN hovering over a sidebar icon THEN a tooltip SHALL show the page name

**Independent Test**: Click each sidebar icon, confirm page switches.

---

### P2: Headless Tauri IPC Layer

**User Story**: As a developer, I want the Tauri event listening and invoke wrappers extracted into a reusable hook/service so each page doesn't re-declare them.

**Why P2**: DRY principle — currently each Svelte file declares its own event listeners.

**Acceptance Criteria**:

1. WHEN a component calls `useTauriEvent("agent:start", callback)` THEN the callback SHALL fire on that event
2. WHEN a component calls `useTauriInvoke("send_prompt", { message })` THEN `invoke` SHALL be called with correct args
3. WHEN the component unmounts THEN event listeners SHALL be cleaned up

**Independent Test**: Create component that uses both hooks, verify in console.

---

## Edge Cases

- WHEN the WebView is not ready THEN all IPC calls SHALL be no-ops or queued
- WHEN a tool call has no result field THEN the expandable section SHALL be hidden
- WHEN the audit log is empty THEN a "No activity yet" empty state SHALL display
- WHEN the proposals list is empty THEN a "No pending mutations" empty state SHALL display
- WHEN `invoke` throws an error THEN the error SHALL be shown in the chat page as an error message
- WHEN messages array is empty THEN a welcome/placeholder message SHALL display

---

## Requirement Traceability

| Requirement ID | Story | Phase | Status |
| -------------- | ----- | ----- | ------ |
| SUM-01 | P1: Chat Page | Design | Pending |
| SUM-02 | P1: Chat Page — streaming render | Design | Pending |
| SUM-03 | P1: Chat Page — tool calls | Design | Pending |
| SUM-04 | P1: Chat Page — approval flow | Design | Pending |
| SUM-05 | P1: Chat Page — markdown rendering | Design | Pending |
| SUM-06 | P1: Status Page | Design | Pending |
| SUM-07 | P1: Proposals Page | Design | Pending |
| SUM-08 | P1: Activity Page | Design | Pending |
| SUM-09 | P2: Sidebar navigation | Design | Pending |
| SUM-10 | P2: Tauri IPC hooks | Design | Pending |

**Coverage**: 10 total, 0 mapped to tasks, 10 unmapped

---

## Success Criteria

- [x] All 4 P1 stories pass their acceptance criteria
- [x] Zero regressions in Tauri IPC — all invoke calls and event listeners work identically
- [x] Visual design matches the AgntOS design system (`docs/design.md`) precisely
- [x] All shadcn/ui components use the correct AgntOS CSS variable tokens
- [x] TypeScript strict mode compiles without errors
