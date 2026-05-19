# Design: Pi + Tauri Migration

## Architecture Overview

```
                    USER FACING (AgntOS branded)
                    ─────────────────────────────
┌──────────────────────────────────────────────────────────────┐
│                AgntOS Control Centre (Tauri)                   │
│                                                               │
│  ┌─────────────────────────────────────────────────────────┐ │
│  │  Svelte Frontend (4 pages)                               │ │
│  │  Chat · Status · Proposals · Activity                    │ │
│  │  Pure AgntOS branding. No Pi references.                  │ │
│  └────────────────────────┬────────────────────────────────┘ │
│                           │ Tauri IPC (invoke + events)       │
│  ┌────────────────────────▼────────────────────────────────┐ │
│  │  Rust Backend                                            │ │
│  │                                                          │ │
│  │  ┌──────────────┐  ┌──────────────────────────────┐     │ │
│  │  │  Pi Bridge    │  │  Commands                     │     │ │
│  │  │               │  │  send_prompt, send_steer,    │     │ │
│  │  │  Spawns Pi    │  │  send_abort, set_model,      │     │ │
│  │  │  with flags:   │  │  get_status, get_system_info│     │ │
│  │  │  --no-builtin  │  │                              │     │ │
│  │  │  --no-context  │  │  Routes approval responses   │     │ │
│  │  │  --system-pr.  │  │                              │     │ │
│  │  │  --extension   │  └──────────────────────────────┘     │ │
│  │  └───────┬───────┘                                        │ │
│  │          │ stdin/stdout JSONL (RPC protocol)               │ │
│  └──────────┼────────────────────────────────────────────────┘ │
└─────────────┼────────────────────────────────────────────────┘
              │
              │       HIDDEN BACKEND (implementation detail)
              │       ──────────────────────────────────────
              ▼
┌─────────────────────────────────────┐      ┌────────────────────┐
│  Pi (Node.js)                        │      │  agntctl (Rust)     │
│                                      │      │                     │
│  --mode rpc (RPC protocol)           │      │  agntctl bash       │
│  --no-builtin-tools                  │◄────►│  agntctl read       │
│  --no-context-files                  │ sub.  │  agntctl write      │
│  --system-prompt <AgntOS prompt>     │      │  agntctl edit       │
│  --extension agntos-tools            │      │  agntctl propose    │
│                                      │      │  agntctl apply      │
│  Only AgntOS tools:                  │      │  agntctl rollback   │
│  agntos_propose, agntos_apply, ...   │      │  agntctl inspect    │
│                                      │      │  agntctl audit      │
│  Agent engine:                       │      │  agntctl memory     │
│  - LLM calls                         │      └────────────────────┘
│  - Tool dispatch                     │
│  - Session trees (JSONL)             │      ┌────────────────────┐
│  - Compaction                        │      │  NixOS System       │
│  - Auto-retry                        │      │  /etc/agntos/       │
│  - Extension UI (approvals)          │      │  nixos-rebuild      │
└─────────────────────────────────────┘      │  audit.jsonl         │
                                             └────────────────────┘
```

## Pi Launch Configuration

Pi is launched with no visible identity. The exact command:

```bash
pi --mode rpc \
   --no-builtin-tools \
   --no-context-files \
   --system-prompt "$(cat /etc/agntos/AGENTS.md)" \
   --extension /etc/agntos/extensions/agntos-tools/index.ts
```

| Flag | Effect |
|------|--------|
| `--mode rpc` | JSONL protocol over stdin/stdout |
| `--no-builtin-tools` | Disables Pi's read/write/edit/bash/grep/find/ls. `agntos_*` tools are the only tools available. |
| `--no-context-files` | Disables AGENTS.md discovery. User's personal Pi config never contaminates AgntOS. |
| `--system-prompt` | Replaces Pi's default system prompt with pure AgntOS instructions. |
| `--extension` | Loads the agntos-tools extension. |

## Component Details

### 1. Tauri App (`agntos-cc`)

**Rust crate:** `crates/agntos-cc/`

```
crates/agntos-cc/
├── Cargo.toml
├── src/
│   ├── main.rs           # Tauri entry point
│   ├── pi_bridge.rs      # Pi subprocess management + RPC parsing
│   ├── commands.rs       # Tauri invoke handlers (frontend → backend)
│   └── config.rs         # Read /etc/agntos/models.toml, Pi flags
├── frontend/
│   ├── package.json
│   ├── src/
│   │   ├── App.svelte       # Main app with tab navigation
│   │   ├── lib/
│   │   │   ├── markdown.js   # Markdown rendering via marked + highlight.js
│   │   │   └── types.js      # Tool metadata (AgntOS branded)
│   │   ├── components/
│   │   │   ├── ChatPage.svelte         # Chat interface
│   │   │   ├── MessageBubble.svelte    # Message, tool call, approval rendering
│   │   │   ├── StatusPage.svelte       # Agent/system status
│   │   │   ├── ProposalsPage.svelte    # Pending proposals
│   │   │   ├── ActivityPage.svelte     # Audit log
│   │   │   └── StatusIndicator.svelte  # Connection + turn state
│   │   └── stores/
│   │       └── index.js     # Svelte stores for messages, connection, proposals
│   └── index.html
├── tauri.conf.json
├── build.rs
├── capabilities/
│   └── default.json
└── icons/
```

### 2. AgntOS System Prompt (`/etc/agntos/AGENTS.md`)

A standalone Markdown file that completely replaces Pi's system prompt. The LLM sees this as its identity:

```markdown
# AgntOS — NixOS System Agent

You are the system agent for AgntOS, an AI-native NixOS distribution.

## Workflow
1. **Inspect** — Check current system state with agntos_inspect
2. **Propose** — Stage changes with agntos_propose
3. **Approval** — User must approve before agntos_apply runs
4. **Apply** — Execute the approved proposal
5. **Verify** — Confirm the change was applied

## Available Tools
- agntos_inspect: Examine CPU, memory, disks, network, services, packages
- agntos_propose: Generate a NixOS configuration change proposal
- agntos_apply: Apply a proposal (requires user confirmation)
- agntos_rollback: Roll back to a previous generation
- agntos_audit: View system mutation history
- agntos_memory: Read/write AgntOS MEMORY.md and USER.md
- agntos_bash: Execute shell commands
- agntos_read: Read file contents
- agntos_write: Write content to a file
- agntos_edit: Edit a file (find and replace)

## Rules
- NEVER apply changes without a proposal first.
- ALWAYS use agntos_inspect before suggesting changes.
- Use agntos_memory to store user preferences and system facts.
- The audit log tracks all mutations. Use agntos_audit to check history.
```

### 3. Pi Extension: `agntos-tools`

**Location:** `/etc/agntos/extensions/agntos-tools/index.ts`

Registers ALL tools the LLM can call via `pi.registerTool()`:

| Tool | agntctl command | Description |
|------|----------------|-------------|
| `agntos_propose` | `agntctl propose --config-dir /etc/agntos "..."` | Generate a NixOS config change proposal |
| `agntos_apply` | `agntctl apply --config-dir /etc/agntos <id>` | Apply a proposal (requires confirmation) |
| `agntos_rollback` | `agntctl rollback apply --config-dir /etc/agntos` | Roll back to previous generation |
| `agntos_inspect` | `agntctl inspect <target>` | Inspect system state |
| `agntos_audit` | `agntctl audit list` / `agntctl audit show <id>` | View audit log |
| `agntos_memory` | `agntctl memory show` / `agntctl memory add` | Read/write MEMORY.md |
| `agntos_bash` | `agntctl bash <command>` | Shell commands (replaces Pi's built-in bash) |
| `agntos_read` | `agntctl read <path>` | Read files (replaces Pi's built-in read) |
| `agntos_write` | `agntctl write <path> --content "..."` | Write files (replaces Pi's built-in write) |
| `agntos_edit` | `agntctl edit <path> --old "..." --new "..."` | Edit files (replaces Pi's built-in edit) |

The `agntos_apply` tool uses Pi's extension UI protocol for confirmation:

```typescript
import type { ExtensionAPI } from "@earendil-works/pi-coding-agent";
import { Type } from "typebox";
import { execSync } from "node:child_process";

export default function (pi: ExtensionAPI) {
  const agntctl = (args: string[]) => {
    const result = execSync(`agntctl ${args.join(" ")}`, {
      encoding: "utf-8",
      maxBuffer: 10 * 1024 * 1024,
    });
    return { content: [{ type: "text", text: result.stdout }] };
  };

  pi.registerTool({
    name: "agntos_inspect",
    label: "AgntOS Inspect",
    description: "Inspect system state — CPU, memory, disks, network, services, packages.",
    parameters: Type.Object({
      target: Type.Optional(Type.String({ description: "Inspect target: system, cpu, memory, disks, network, gpu, services" })),
    }),
    async execute(toolCallId, params, signal, onUpdate, ctx) {
      return agntctl(["inspect", params.target || "system"]);
    },
  });

  pi.registerTool({
    name: "agntos_apply",
    label: "AgntOS Apply",
    description: "Apply a NixOS proposal. Requires user confirmation.",
    parameters: Type.Object({
      proposalId: Type.String({ description: "ID of the proposal to apply" }),
    }),
    promptGuidelines: [
      "Use agntos_apply only after user has reviewed and approved the proposal.",
      "Never call agntos_apply without first presenting the proposal to the user.",
    ],
    async execute(toolCallId, params, signal, onUpdate, ctx) {
      const approved = await ctx.ui.confirm({
        title: `Apply proposal: ${params.proposalId}?`,
        message: "This will modify your NixOS configuration and trigger a rebuild.",
      });
      if (!approved) {
        return { content: [{ type: "text", text: "User rejected the proposal." }], isError: true };
      }
      return agntctl(["apply", "--config-dir", "/etc/agntos", params.proposalId]);
    },
  });

  // ... similar for all other agntos_* tools
}
```

### 4. Pi Bridge (`pi_bridge.rs`)

The bridge manages the Pi subprocess and translates between Pi RPC and Tauri events.

```rust
// Key operations:
impl PiBridge {
    async fn start() -> Result<Self>;
        // Spawns: pi --mode rpc --no-builtin-tools --no-context-files
        //         --system-prompt <AgntOS prompt> --extension <agntos-tools>
    async fn send_prompt(&self, msg: &str) -> Result<()>;
    async fn send_steer(&self, msg: &str) -> Result<()>;
    async fn send_abort(&self) -> Result<()>;
    async fn set_model(&self, provider: &str, model: &str) -> Result<()>;
    async fn new_session(&self) -> Result<()>;
    async fn switch_session(&self, path: &str) -> Result<()>;
    async fn send_extension_ui_response(&self, id: &str, confirmed: bool) -> Result<()>;
}
```

**Event routing — all Pi-referenced event names are internal only:**

```
Pi stdout JSON → parse → match:
  message_update    → Tauri event: "agent:message-update"
  tool_execution_start → "agent:tool-start"
  tool_execution_end   → "agent:tool-end"
  agent_end            → "agent:end"
  agent_start          → "agent:start"
  extension_ui_request → "agent:approval-request"
  error                → "agent:error"
```

Note: All Tauri events are prefixed with `agent:` not `pi:`. No Pi branding leaks to the frontend.

### 5. agntctl Integration

agntctl already has all needed subcommands:

| Subcommand | File | Usage |
|------------|------|-------|
| `bash` | `sys.rs:54` | `agntctl bash "ls -la"` |
| `read` | `sys.rs:10` | `agntctl read /path/to/file` |
| `write` | `sys.rs:16` | `agntctl write /path/to/file --content "..."` |
| `edit` | `sys.rs:29` | `agntctl edit /path/to/file --old "..." --new "..."` |
| `propose` | `propose.rs` | `agntctl propose "install nginx"` |
| `apply` | `apply.rs` | `agntctl apply --config-dir /etc/agntos <id>` |
| `rollback` | `rollback.rs` | `agntctl rollback apply` |
| `inspect` | `inspect.rs` | `agntctl inspect system` |
| `audit` | `audit.rs` | `agntctl audit list` |
| `memory` | `memory.rs` | `agntctl memory show` |

### 6. Nix Module Changes

Pi is a private dependency. The Nix module wraps it — `pi` is not exposed to the user's PATH unless they explicitly opt in.

```nix
# NEW: Pi as hidden dependency
# Pi is installed but not symlinked to bin/pi
environment.systemPackages = [ pkgs.agntos-cc pkgs.agntctl ];

# The Tauri app manages Pi internally, finding it via Nix store path
services.agntos = {
  enable = true;
  piPackage = pkgs.nodePackages.pi-coding-agent;  # hidden dep
  model = "ollama/qwen3:6b";
};

# Optional: expose Pi to user's PATH for standalone use
programs.pi.enable = true;  # separate concern
```

### 7. Frontend Design (Svelte)

**Layout:** Sidebar navigation (AgntOS branded):

```
┌─────────────────────────────────────────┐
│  AgntOS Control Centre                   │
├──────┬──────────────────────────────────┤
│  💬  │                                   │
│ Chat │  [Chat interface with markdown]   │
│      │                                   │
│  📊  │  Agent is thinking...             │
│Status│                                   │
│      │  ┌─ Proposal ──────────────┐      │
│  📋  │  │ Install nginx           │      │
│Props │  │ Approve  ·  Dismiss      │      │
│      │  └─────────────────────────┘      │
│  📜  │                                   │
│ Audit│  [Ask the agent...]        [Send] │
├──────┴──────────────────────────────────┤
│  ● Connected  ·  Qwen3-6-35B  ·  3s    │
└─────────────────────────────────────────┘
```

**Zero Pi references** in UI. Status says "Connected" not "Connected to Pi."

### 8. Data Flow: Chat Message

```
User types "install nginx" in Svelte
  → Svelte calls invoke("send_prompt", { message: "install nginx" })
  → Rust backend sends {"type":"prompt","message":"install nginx"} to Pi stdin
  → Pi processes (no Pi branding visible)
  → Pi emits events on stdout:
      {"type":"agent_start"}
      {"type":"message_update","assistantMessageEvent":{"type":"text_delta","delta":"Let me check..."}}
      {"type":"tool_execution_start","toolName":"agntos_inspect",...}
  → Bridge receives events, re-emits as "agent:*" Tauri events
  → Svelte renders: markdown text, tool call cards, approval cards
  → User approves
  → Svelte calls invoke("send_extension_ui_response", { id, confirmed: true })
  → Rust sends {"type":"extension_ui_response","id":"...","confirmed":true} to Pi stdin
  → Pi extension receives confirmation, calls agntctl apply
  → Result streams back
  → {"type":"agent_end","messages":[...]}
```

### 9. Session Persistence

- **Conversation sessions:** Pi JSONL in `~/.pi/agent/sessions/`. `new_session` RPC creates new files, `switch_session` loads existing ones.
- **System audit:** `agntctl audit` writes to `/var/log/agntos/audit.jsonl`. Only mutations (propose, apply, rollback) — no chat history.
- One-off prompts (e.g., "check disk usage") use `--no-session` or just `new_session` + discard.

### 10. Comparison: Before vs After

| Aspect | Before (current) | After (Pi + Tauri) |
|--------|-------------------|---------------------|
| Agent engine | agntd (Rust, ~3600 lines, custom) | Pi (Node.js, battle-tested, hidden) |
| GUI framework | QML/Kirigami (~2800 lines) | Svelte in Tauri webview (~1500 lines) |
| Agent identity | AgntOS (but leaks via system prompt) | PURELY AgntOS — no Pi reference anywhere |
| User-facing tools | Custom Rust implementations | agntos_* tools via Pi extension → agntctl |
| IPC protocol | Custom socket + JSON handshake | Pi RPC (JSONL stdin/stdout) |
| Markdown | `Text.MarkdownText` (QML, flaky) | `marked` + `highlight.js` (native web) |
| Approval flow | Custom Mutex-based gate | Pi extension UI protocol |
| Session persist | SQLite (agntd) | Pi JSONL for chat, agntctl audit for mutations |
| Total custom code | ~7300 lines Rust + ~900 QML | ~1300 lines Rust + ~1500 Svelte + ~500 TS |
| Platform issues | Qt/Wayland/QML (constant) | None (webview is universal) |
| Streaming | Custom poll-based timer | Native Pi message_update events |
| Auto-retry | None | Built-in Pi |
| Compaction | None | Built-in Pi |
| Session trees | None | Built-in Pi |
| Pi on user's PATH | N/A (agntd doesn't use Pi) | Only if user opts in to programs.pi |
