# shadcn/ui Frontend — Design

**Spec**: `.specs/features/shadcn-ui-migration/spec.md`
**Status**: Draft

---

## Architecture Overview

The migration replaces only the frontend layer. The Tauri Rust backend (`crates/agntos-cc/src/`) is untouched.

```
Before:                    After:
┌──────────────────┐       ┌──────────────────┐
│  Svelte 5 + CSS  │       │  React + TS      │
│  ChatPage.svelte │       │  shadcn/ui        │
│  StatusPage      │       │  ChatPage.tsx     │
│  ProposalsPage   │  →    │  StatusPage.tsx   │
│  ActivityPage    │       │  ProposalsPage    │
│  stores/index.js │       │  ActivityPage     │
│  lib/*.js        │       │  hooks/*.ts       │
│                  │       │  lib/*.ts         │
├──────────────────┤       ├──────────────────┤
│  Tauri IPC       │       │  Tauri IPC        │
│  (invoke/listen) │       │  (invoke/listen)  │
├──────────────────┤       ├──────────────────┤
│  Rust Backend    │       │  Rust Backend     │
│  pi_bridge.rs    │       │  pi_bridge.rs     │
│  commands.rs     │       │  commands.rs      │
└──────────────────┘       └──────────────────┘
```

### Data Flow

```
User Action → React Component → useTauriInvoke() → Tauri invoke() → Rust command handler
                                                                         ↓
React Component ← useTauriEvent() ← Tauri event ← Rust event emission
```

---

## Code Reuse Analysis

### What Stays (unchanged)

| File | Purpose |
| ---- | ------- |
| `src/main.rs` | Tauri entry point, bridge setup, invoke handler registration |
| `src/pi_bridge.rs` | Pi subprocess management + RPC parsing |
| `src/commands.rs` | All invoke handlers (send_prompt, list_proposals, etc.) |
| `src/config.rs` | App config loading |
| `tauri.conf.json` | Tauri configuration |
| `Cargo.toml` | Rust dependencies |
| `build.rs` | Tauri build script |
| `capabilities/default.json` | Tauri permissions |

### What Gets Replaced

| Current (Svelte) | Replace With (React + shadcn/ui) |
| ----------------- | -------------------------------- |
| `src/main.js` | `src/main.tsx` |
| `src/App.svelte` | `src/App.tsx` |
| `src/components/ChatPage.svelte` | `src/components/ChatPage.tsx` |
| `src/components/MessageBubble.svelte` | `src/components/ChatMessage.tsx` |
| `src/components/InlineStatus.svelte` | Inline in ChatPage (simplified) |
| `src/components/StatusPage.svelte` | `src/components/StatusPage.tsx` |
| `src/components/ProposalsPage.svelte` | `src/components/ProposalsPage.tsx` |
| `src/components/ActivityPage.svelte` | `src/components/ActivityPage.tsx` |
| `src/stores/index.js` | `src/hooks/useAgentStore.ts` |
| `src/lib/markdown.js` | `src/lib/markdown.ts` |
| `src/lib/types.js` | `src/lib/types.ts` |
| `src/lib/intent.js` | `src/lib/intent.ts` |
| `src/app.css` | `src/index.css` (shadcn/ui style) |
| `package.json` (Svelte deps) | `package.json` (React + shadcn/ui deps) |
| `svelte.config.js` | Removed |
| `vite.config.js` | `vite.config.ts` (React plugin) |

### What We Reuse (ported to TypeScript)

| Current | Ports To |
| ------- | -------- |
| `lib/markdown.js` | `lib/markdown.ts` — same `marked` + `highlight.js` logic |
| `lib/types.js` | `lib/types.ts` — same tool metadata |
| `lib/intent.js` | `lib/intent.ts` — same intent extraction |

---

## Components

### App (Root)

- **Purpose**: Root component with sidebar navigation, page routing, and connection status bar
- **Location**: `frontend/src/App.tsx`
- **Interfaces**: None (root component)
- **Dependencies**: All page components, sidebar component
- **Reuses**: shadcn/ui `TooltipProvider`, `Separator`

### Sidebar

- **Purpose**: Vertical icon navigation with tooltips for page switching
- **Location**: `frontend/src/components/Sidebar.tsx`
- **Interfaces**: `{ activePage: Page, onNavigate: (page: Page) => void }`
- **Dependencies**: None (pure presentational)
- **Reuses**: shadcn/ui `Tooltip`, `TooltipTrigger`, `TooltipContent`
- **Icons**: Lucide `MessageSquare`, `Info`, `FileText`, `Clock`

### ChatPage

- **Purpose**: Agent chat interface — message list, streaming text, tool calls, approval cards, input
- **Location**: `frontend/src/components/ChatPage.tsx`
- **Interfaces**: None (reads from Tauri events via hooks)
- **Dependencies**: `TauriEventProvider`, `useAgentStore`, `ChatMessage`, `ChatInput`
- **Reuses**: shadcn/ui `ScrollArea`, `Button`, `Textarea`

### ChatMessage

- **Purpose**: Renders a single message — user bubble, agent bubble, tool call card, approval card, or error
- **Location**: `frontend/src/components/ChatMessage.tsx`
- **Interfaces**: `{ message: ChatEntry, onApprove: (id: string) => void, onReject: (id: string) => void }`
- **Dependencies**: `lib/markdown.ts`
- **Reuses**: shadcn/ui `Card`, `Badge`, `Button`, `Separator`

### ChatInput

- **Purpose**: Text input with Send button, Enter-to-send, Shift+Enter-newline
- **Location**: `frontend/src/components/ChatInput.tsx`
- **Interfaces**: `{ onSend: (text: string) => void, disabled: boolean }`
- **Dependencies**: None
- **Reuses**: shadcn/ui `Textarea`, `Button`

### StatusPage

- **Purpose**: Agent connection state, system info, watchdog health displayed in a card grid
- **Location**: `frontend/src/components/StatusPage.tsx`
- **Interfaces**: None (reads from hooks + invoke)
- **Dependencies**: `useAgentStore`, `useTauriInvoke`
- **Reuses**: shadcn/ui `Card`, `CardHeader`, `CardContent`, `Badge`

### ProposalsPage

- **Purpose**: Pending and applied proposals with apply/dismiss/revert actions
- **Location**: `frontend/src/components/ProposalsPage.tsx`
- **Interfaces**: None (calls invoke directly)
- **Dependencies**: `useTauriInvoke`
- **Reuses**: shadcn/ui `Card`, `Badge`, `Button`, `Separator`

### ActivityPage

- **Purpose**: Searchable audit log with entry details and rollback actions
- **Location**: `frontend/src/components/ActivityPage.tsx`
- **Interfaces**: None (calls invoke directly)
- **Dependencies**: `useTauriInvoke`
- **Reuses**: shadcn/ui `Card`, `Badge`, `Button`, `Input`

### ThinkingIndicator

- **Purpose**: Animated spinner/thinking bar shown while agent processes
- **Location**: `frontend/src/components/ThinkingIndicator.tsx`
- **Interfaces**: `{ state: 'thinking' | 'working' | 'hidden', elapsed: number, content?: string }`
- **Dependencies**: None
- **Reuses**: shadcn/ui `Badge`

---

## Hooks (replacing Svelte stores)

### useAgentStore

- **Purpose**: Global state for connection status, messages, model info
- **Location**: `frontend/src/hooks/useAgentStore.ts`
- **Interfaces**: `{ connection, messages, dispatch }` — React context-based
- **Dependencies**: None

### useTauriEvent

- **Purpose**: Subscribe to Tauri events with automatic cleanup on unmount
- **Location**: `frontend/src/hooks/useTauriEvent.ts`
- **Interfaces**: `useTauriEvent(event: string, callback: (payload: any) => void): void`
- **Dependencies**: `window.__TAURI__.event.listen`

### useTauriInvoke

- **Purpose**: Typed wrapper around `window.__TAURI__.core.invoke`
- **Location**: `frontend/src/hooks/useTauriInvoke.ts`
- **Interfaces**: `useTauriInvoke<T>(cmd: string, args?: Record<string, unknown>): { data: T | null, error: Error | null, loading: boolean, execute: () => Promise<T> }`
- **Dependencies**: `window.__TAURI__.core.invoke`

---

## AgntOS → shadcn/ui Theme Mapping

shadcn/ui uses CSS variables for theming. AgntOS brand tokens are mapped as follows:

```css
@import url('https://fonts.googleapis.com/css2?family=Plus+Jakarta+Sans:wght@400;500;600;700&family=Syne:wght@600;700;800&display=swap');

@layer base {
  :root {
    --font-display: 'Syne', sans-serif;
    --font-body: 'Plus Jakarta Sans', sans-serif;
    --font-mono: 'GeistMono Nerd Font', 'Geist Mono', monospace;
  }

  .dark {
    --background: #141416;
    --foreground: #EBEBEC;
    --card: #242428;
    --card-foreground: #EBEBEC;
    --popover: #2C2C31;
    --popover-foreground: #EBEBEC;
    --primary: #F57C48;
    --primary-foreground: #141416;
    --secondary: #1C1C1F;
    --secondary-foreground: #EBEBEC;
    --muted: #1C1C1F;
    --muted-foreground: #9C9CA3;
    --accent: #F57C48;
    --accent-foreground: #141416;
    --destructive: #E5534B;
    --destructive-foreground: #EBEBEC;
    --success: #4CAF7A;
    --warning: #E6A23C;
    --info: #4493F8;
    --border: #333338;
    --input: #333338;
    --ring: #F57C48;
    --radius: 0.625rem;
  }
}
```

---

## Tech Decisions

| Decision | Choice | Rationale |
| -------- | ------ | --------- |
| State management | React Context + useReducer | Simple enough — no need for zustand/redux for 4 pages |
| Routing | State-based (activePage state) | Only 4 pages, no URL routing needed in Tauri webview |
| shadcn/ui install | `npx shadcn@latest init` + add per component | Standard shadcn/ui workflow |
| Tailwind v4 | Yes | Already used in current app via Tailwind v4 |
| Markdown | `marked` + `highlight.js` (same libs) | Zero behavioral change — just port to TypeScript |
| Icons | `lucide-react` | Official shadcn/ui pairing |
| CSS approach | Tailwind classes + shadcn/ui `cn()` utility | Standard shadcn/ui pattern |
