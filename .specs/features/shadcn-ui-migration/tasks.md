# shadcn/ui Frontend — Tasks

**Design**: `.specs/features/shadcn-ui-migration/design.md`
**Status**: Draft

---

## Execution Plan

### Phase 1: Foundation (Sequential)

Project scaffold, theming, IPC hooks, and lib port — everything pages depend on.

```

T1 → T2 → T3 → T4

```

### Phase 2: Pages (Parallel)

Once foundation is laid, all pages can be built independently.

```
            ┌→ T6 (ChatPage) ─┐
            │                  │
T4 ──→ T5 ──┼→ T7 (StatusPage) ┼──→ T10
            │                  │
            ├→ T8 (Proposals) ─┘
            │
            └→ T9 (Activity) ──┘
```

### Phase 3: Integration (Sequential)

Final wiring, test all IPC, remove old files.

```

T10 (Final wiring + cleanup)

```

---

## Task Breakdown

### T1: Scaffold React + shadcn/ui + Tailwind v4

**What**: Initialize the frontend project with React, TypeScript, Vite, Tailwind CSS v4, shadcn/ui, and lucide-react. Replace all Svelte dependencies.

**Where**: `crates/agntos-cc/frontend/`

**Depends on**: None

**Reuses**: `vite.config.js` structure (keep Tauri-specific parts), `index.html` (keep shell)

**Requirement**: SUM-01

**Done when**:

- [ ] `package.json` updated: Svelte deps removed, React deps added (react, react-dom, @types/react, @types/react-dom, @vitejs/plugin-react)
- [ ] `npx shadcn@latest init` completes with dark mode, CSS variables, Tailwind v4
- [ ] `vite.config.ts` uses `@vitejs/plugin-react` instead of `@sveltejs/vite-plugin-svelte` (keep Tauri inline-css plugin)
- [ ] `svelte.config.js` deleted
- [ ] `index.html` updated: load script from `/src/main.tsx`, remove Svelte-specific content
- [ ] `tsconfig.json` created with strict mode
- [ ] `lucide-react` installed
- [ ] `src/main.tsx` created: renders `<App />` into `#app`
- [ ] Directory structure created: `src/components/`, `src/hooks/`, `src/lib/`
- [ ] `cargo check -p agntos-cc` passes (if Rust changes needed)
- [ ] `npm run build` succeeds

**Tests**: none (scaffold)
**Gate**: build

---

### T2: Create AgntOS theme with shadcn/ui CSS variables

**What**: Define the AgntOS brand tokens as shadcn/ui CSS variables in `src/index.css`. Theme must match `docs/design.md` exactly.

**Where**: `frontend/src/index.css`

**Depends on**: T1

**Reuses**: Color values from `docs/design.md`, typography from design doc

**Requirement**: SUM-01

**Done when**:

- [ ] All CSS variables from design.md are defined (--background, --foreground, --primary, --accent, etc.)
- [ ] Syne, Plus Jakarta Sans, and Geist Mono font imports are included
- [ ] Custom `--font-display`, `--font-body`, `--font-mono` CSS variables defined
- [ ] Global styles match existing app: full viewport, no margin/padding, antialiased
- [ ] `prefers-reduced-motion` respected
- [ ] Dark class only (no light mode)
- [ ] Scrollbar styles defined matching AgntOS palette
- [ ] `npx shadcn@latest add button card badge input textarea separator scroll-area tooltip` all install and accept the theme
- [ ] `npm run build` succeeds

**Tests**: none (styles)
**Gate**: build

---

### T3: Create Tauri IPC hooks + agent store

**What**: Create `useTauriEvent`, `useTauriInvoke`, and `useAgentStore` hooks that wrap `window.__TAURI__` APIs with React context, typed invoke wrappers, and automatic cleanup on unmount.

**Where**: `frontend/src/hooks/`

**Files**:

- `hooks/useTauriEvent.ts`
- `hooks/useTauriInvoke.ts`
- `hooks/useAgentStore.ts`
- `hooks/TauriProvider.tsx`

**Depends on**: T1

**Reuses**: Event patterns from existing `App.svelte` and `ChatPage.svelte`

**Requirement**: SUM-10

**Done when**:

- [ ] `useTauriEvent(event, callback)` subscribes via `window.__TAURI__.event.listen` and unsubscribes on unmount
- [ ] `useTauriInvoke(cmd, args)` returns `{ data, error, loading, execute }` with proper TypeScript generics
- [ ] `useAgentStore` exposes React context with: `connection`, `messages`, `dispatch`
- [ ] Connection state includes: `connected`, `model`, `state` (idle/thinking/working)
- [ ] Message type union defined: `'user' | 'assistant' | 'tool' | 'approval' | 'error'`
- [ ] Event wiring matches current behavior:
  - `agent:start` → state = "thinking", clear partial
  - `agent:end` → state = "idle", flush partial
  - `agent:message-update` → handle `text_delta`, `tool_start`, `tool_end`, `approval_request`
  - `agent:connected` / `agent:disconnected` → connection state
  - `agent:error` → push error message
- [ ] `TauriProvider` wraps the app and provides context
- [ ] TypeScript strict mode compiles without errors
- [ ] `npm run build` succeeds

**Tests**: none (Tauri APIs unavailable in CI)
**Gate**: build

---

### T4: Port lib utilities to TypeScript

**What**: Port the existing JavaScript utilities to TypeScript: markdown rendering, tool metadata, and intent extraction.

**Where**: `frontend/src/lib/`

**Files**:

- `lib/markdown.ts` — same `marked` + `highlight.js` + Geist Mono styling
- `lib/types.ts` — tool metadata with AgntOS branding
- `lib/intent.ts` — intent extraction and elapsed time formatting

**Depends on**: T1

**Reuses**: Exact logic from `lib/markdown.js`, `lib/types.js`, `lib/intent.js`

**Requirement**: SUM-02, SUM-03

**Done when**:

- [ ] `markdown.ts` exports `renderMarkdown(text: string): string` using `marked` + `highlight.js` with nix, rust, python, bash, json, javascript language registration
- [ ] `types.ts` exports `TOOLS` map and `getToolMeta(name: string)` with AgntOS-branded colors (orange for propose, cyan for bash, etc.)
- [ ] `intent.ts` exports `extractIntent(toolName, args, thinkingText, textContent)`, `firstSentence(text, maxLen)`, `formatElapsed(secs)`
- [ ] All types are strict — no `any` or implicit `any`
- [ ] `npm run build` succeeds

**Tests**: none (same logic, just porting)
**Gate**: build

---

### T5: Create App layout with Sidebar

**What**: Root app component with sidebar navigation and page routing via state. Connection status in top bar.

**Where**: `frontend/src/App.tsx`, `frontend/src/components/Sidebar.tsx`

**Depends on**: T3, T4

**Reuses**: shadcn/ui `Tooltip`, `TooltipTrigger`, `TooltipContent`, `Separator`; Lucide `MessageSquare`, `Activity`, `FileText`, `Clock`, `User`

**Requirement**: SUM-09

**Done when**:

- [ ] `Sidebar` renders vertical icon nav with 4 items: Chat, Status, Proposals, Activity
- [ ] Active page icon shows orange accent (`#F57C48`) with subtle background
- [ ] Hover tooltips show page name
- [ ] `App.tsx` renders `TauriProvider` wrapping sidebar + page content
- [ ] Page switching via `activePage` state renders the correct component
- [ ] Top bar shows "AgntOS" brand in Syne font, connection status dot (green/red), model name
- [ ] Layout matches mockup: fixed sidebar, scrollable content
- [ ] `npm run build` succeeds

**Tests**: none (visual/layout)
**Gate**: build

---

### T6: Create ChatPage + ChatMessage + ChatInput + ThinkingIndicator [P]

**What**: Full chat interface — message list with auto-scroll, streaming text rendering, tool call cards, approval cards, error messages, and text input with Enter-to-send.

**Where**: `frontend/src/components/`

**Files**:

- `components/ChatPage.tsx`
- `components/ChatMessage.tsx`
- `components/ChatInput.tsx`
- `components/ThinkingIndicator.tsx`

**Depends on**: T3, T4, T5

**Reuses**:

- shadcn/ui `ScrollArea`, `Button`, `Textarea`, `Card`, `Badge`, `Separator` (from T2)
- Lucide icons: `Send`, `Loader2`, `Check`, `X`, `AlertTriangle`, `ChevronDown`, `ChevronRight`, `Terminal`, `FileEdit`
- `lib/markdown.ts`, `lib/types.ts`, `lib/intent.ts` (from T4)

**Requirement**: SUM-01, SUM-02, SUM-03, SUM-04, SUM-05

**Done when**:

- [ ] `ChatPage` listens to agent Tauri events and renders message list
- [ ] `ChatInput` has textarea + Send button; Enter sends, Shift+Enter newlines
- [ ] `ChatMessage` renders 5 message types: user bubble, agent bubble, tool card, approval card, error
- [ ] User messages: orange background, right-aligned, no icon
- [ ] Agent messages: `chat-bubble-agent` styling, render via `renderMarkdown()`, blinking cursor during streaming
- [ ] Tool call cards: left border colored per tool type, name in monospace, spinner while running, checkmark when done, expandable result section
- [ ] Approval cards: warning border, title, message, Approve/Dismiss buttons that call `useTauriInvoke`
- [ ] Error messages: red styling
- [ ] `ThinkingIndicator` shows spinner + text during thinking state with elapsed time counter
- [ ] Auto-scroll to bottom on new messages
- [ ] Empty state with welcome message when no messages exist
- [ ] Input disabled when agent is not idle
- [ ] `npm run build` succeeds

**Tests**: none (Tauri APIs unavailable in CI)
**Gate**: build

---

### T7: Create StatusPage [P]

**What**: Three-column card grid showing Agent connection status, System info, and Watchdog health.

**Where**: `frontend/src/components/StatusPage.tsx`

**Depends on**: T3, T5

**Reuses**:

- shadcn/ui `Card`, `CardHeader`, `CardContent`, `Badge` (from T2)
- Lucide icons: `Circle`, `Monitor`, `Cpu`, `HardDrive`, `ShieldAlert`
- `useAgentStore` and `useTauriInvoke`

**Requirement**: SUM-06

**Done when**:

- [ ] Agent card shows: connected/disconnected status (colored dot), state, model name (monospace), profile name
- [ ] System card shows: CPU, RAM, Disk usage, failed units count (green if 0, red if >0)
- [ ] Watchdog card shows: alert count, last check time, health status
- [ ] Empty/loading states handled gracefully
- [ ] Grid layout: 3 columns on wide, stacks on narrow
- [ ] `npm run build` succeeds

**Tests**: none
**Gate**: build

---

### T8: Create ProposalsPage [P]

**What**: Two-section page: pending proposals (with apply/dismiss) and applied mutations (with revert).

**Where**: `frontend/src/components/ProposalsPage.tsx`

**Depends on**: T3, T5

**Reuses**:

- shadcn/ui `Card`, `Badge`, `Button`, `Separator` (from T2)
- Lucide icons: `CheckCircle`, `XCircle`, `Undo2`
- `useTauriInvoke`

**Requirement**: SUM-07

**Done when**:

- [ ] Pending section: each proposal shows ID (monospace), prompt text, status badge (amber), Apply + Dismiss buttons
- [ ] Applied section: each entry shows ID, description, status badge (green), Revert button
- [ ] Apply calls `invoke("apply_proposal", { id })` and refreshes list
- [ ] Dismiss removes from local list
- [ ] Revert calls `invoke("rollback_to", { generation })` and refreshes
- [ ] Empty states for both sections
- [ ] `npm run build` succeeds

**Tests**: none
**Gate**: build

---

### T9: Create ActivityPage [P]

**What**: Searchable audit log with action entries and rollback buttons.

**Where**: `frontend/src/components/ActivityPage.tsx`

**Depends on**: T3, T5

**Reuses**:

- shadcn/ui `Card`, `Badge`, `Button`, `Input` (from T2)
- Lucide icons: `CheckCircle`, `FileEdit`, `RotateCcw`, `Search`
- `useTauriInvoke`

**Requirement**: SUM-08

**Done when**:

- [ ] Loads 50 most recent entries via `invoke("list_audit_entries")`
- [ ] Each entry card shows: icon (apply=green check, propose=amber file, rollback=red rotate), action name, description, timestamp, audit ID
- [ ] Search input filters entries by text match in real-time
- [ ] Apply entries show Revert button calling `invoke("rollback_to")`
- [ ] Empty state when no entries
- [ ] Loading state during fetch
- [ ] Refresh button reloads all entries
- [ ] `npm run build` succeeds

**Tests**: none
**Gate**: build

---

### T10: Final wiring, clean up old Svelte files, verify

**What**: Connect all pages through App.tsx, remove all old Svelte files, update Vite config, verify end-to-end.

**Where**: `frontend/`

**Depends on**: T5, T6, T7, T8, T9

**Reuses**: Nothing new — final integration pass

**Requirement**: All SUM requirements

**Done when**:

- [ ] `App.tsx` renders all 4 pages via activePage state
- [ ] All old Svelte files deleted: `App.svelte`, all `*.svelte` components, `stores/`, `app.css`, `main.js`
- [ ] `vite.config.ts` confirmed using `@vitejs/plugin-react` only
- [ ] `svelte.config.js` deleted
- [ ] `index.html` references `/src/main.tsx`
- [ ] `package.json` has no Svelte dependencies
- [ ] `npm run build` succeeds with zero warnings
- [ ] `npm run dev` launches Vite dev server without errors
- [ ] Verify all 10 IPC invoke commands from the current app are still callable:
  - `send_prompt`, `send_steer`, `send_abort`, `set_model`
  - `new_session`, `switch_session`, `send_extension_ui_response`
  - `get_connection_status`, `get_available_models`, `get_system_info`
  - `list_proposals`, `apply_proposal`, `list_audit_entries`, `rollback_to`
- [ ] Verify all 8 Tauri events still handled:
  - `agent:connected`, `agent:disconnected`
  - `agent:start`, `agent:end`
  - `agent:message-update`, `agent:tool-start`, `agent:tool-end`
  - `agent:approval-request`, `agent:error`, `agent:rpc-response`

**Tests**: none (Tauri unavailable in CI)
**Gate**: build

---

## Parallel Execution Map

```

Phase 1 (Sequential — foundation must be laid first):
  T1 (scaffold) ──→ T2 (theme) ──→ T3 (hooks) ──→ T4 (lib port)
                                                      │
Phase 2 (T5 needs T3+T4; T6-T9 additionally need T5):
                                                      ↓
                                              T5 (App layout)
                                              ╱  ╱  │  ╲  ╲
                                             ↓   ↓   ↓   ↓   ↓
                                           T6   T7   T8   T9
                                           [P]  [P]  [P]  [P]
                                              ╲  ╲  │  ╱  ╱
Phase 3 (Sequential — all pages done):          ↓   ↓
                                              T10 (wiring + cleanup)

```

**Parallelism constraint**: T6-T9 are marked `[P]` — they have no shared mutable state (each is its own page component with independent data fetching). They depend on T5 (layout) which must complete first for visual integration, but the component logic itself has no cross-dependencies.

---

## Task Granularity Check

| Task | Scope | Status |
| ---- | ----- | ------ |
| T1: Scaffold React + shadcn/ui | 1 project configuration | ✅ Granular |
| T2: Create AgntOS theme CSS | 1 CSS file | ✅ Granular |
| T3: Create Tauri IPC hooks | 4 hook files (cohesive module) | ✅ Granular |
| T4: Port lib utilities | 3 lib files (cohesive module) | ✅ Granular |
| T5: App layout + Sidebar | 2 component files | ✅ Granular |
| T6: ChatPage + messages + input | 4 component files (single page) | ✅ Granular |
| T7: StatusPage | 1 component file | ✅ Granular |
| T8: ProposalsPage | 1 component file | ✅ Granular |
| T9: ActivityPage | 1 component file | ✅ Granular |
| T10: Wiring + cleanup | 1 integration pass | ✅ Granular |

---

## Diagram-Definition Cross-Check

| Task | Depends On (body) | Diagram Shows | Status |
| ---- | ----------------- | ------------- | ------ |
| T1 | None | T1 root | ✅ Match |
| T2 | T1 | T2 after T1 | ✅ Match |
| T3 | T1 | T3 after T1 | ✅ Match |
| T4 | T1 | T4 after T1 | ✅ Match |
| T5 | T3, T4 | T5 after T4 | ✅ Match |
| T6 | T3, T4, T5 | T6 after T5 | ✅ Match |
| T7 | T3, T5 | T7 after T5 | ✅ Match |
| T8 | T3, T5 | T8 after T5 | ✅ Match |
| T9 | T3, T5 | T9 after T5 | ✅ Match |
| T10 | T5, T6, T7, T8, T9 | T10 after all | ✅ Match |

---

## Test Co-location Validation

No TESTING.md exists for the frontend crate. The existing Svelte frontend has zero tests. All tasks gate on `build` only. No test co-location violations.

| Task | Code Layer | Tests | Gate | Status |
| ---- | ---------- | ----- | ---- | ------ |
| T1 | Scaffold | none | build | ✅ No test req |
| T2 | CSS | none | build | ✅ No test req |
| T3 | Hooks | none | build | ✅ No test req |
| T4 | Lib | none | build | ✅ No test req |
| T5 | Components | none | build | ✅ No test req |
| T6 | Components | none | build | ✅ No test req |
| T7 | Components | none | build | ✅ No test req |
| T8 | Components | none | build | ✅ No test req |
| T9 | Components | none | build | ✅ No test req |
| T10 | Integration | none | build | ✅ No test req |
