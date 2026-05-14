# AgntOS Foundation Design

## Overview

AgntOS is structured as a NixOS flake with Rust system tools and a Plasma/Kirigami user experience. The foundation separates distro composition, OS control, agent runtime, model routing, and UI so each can evolve independently.

```mermaid
flowchart TD
  User[User] --> LLM[LLM (user-configured endpoint)]
  LLM --> Agent[agntd agent daemon]
  Agent --> Memory[Memory: MEMORY.md + USER.md]
  Agent --> Tools[Tool execution: agntctl]
  Tools --> OSCTL[agntctl CLI]
  OSCTL --> NixConfig[AgntOS-managed Nix config]
  NixConfig --> Rebuild[NixOS rebuild / VM / ISO]
  Agent --> Router[model routing layer]
  Router --> Cloud[cloud endpoints]
  Router --> Local[local model backends]
  Memory --> SessionSearch[SQLite FTS5 for historical queries]
  Agent --> Audit[JSONL audit log]
```

### Data Flow

1. **Session start**: agent loads `MEMORY.md` + `USER.md` as frozen snapshot
2. **User input**: sent to LLM with system prompt (memory + profile + tools)
3. **LLM responds** with text and/or tool calls
4. **Tool calls** → `agntctl` subprocess → result fed back to LLM
5. **LLM produces final response** using tool results
6. **Memory updates**: agent may call `memory` tool to add/update facts
7. **Turn saved** to session store (SQLite FTS5)
8. **Destructive ops** (apply): confirmation prompt before execution

### Hermes-Style Memory Layer

Memory is the core architectural difference from a generic chatbot. Two levels:

| Level | Storage | Capacity | Availability | Use case |
|---|---|---|---|---|
| **Core memory** | `MEMORY.md`, `USER.md` | ~3,500 chars total | Always in system prompt | System facts, preferences, lessons |
| **Session search** | SQLite FTS5 | Unlimited | On-demand | "When did we fix DNS?" |

Core memory is:
- **Agent-curated**: the agent adds/replaces/removes facts via `memory` tool
- **Bounded**: hard character caps force curation > quality
- **Frozen snapshot**: loaded once per session, changes persist to disk for next session
- **Security-scanned**: injection, exfiltration, invisible Unicode detected on every write

This is deliberately NOT a vector database or RAG system. The facts an OS agent needs (GPU model, disk layout, package list) are small, stable, and always relevant. A vector DB adds latency, complexity, and failure modes without providing value.

## Repository Shape

Initial target structure:

```text
agntos/
  flake.nix
  flake.lock
  modules/
    agntos/
      base.nix
      desktop-plasma.nix
      agent.nix
      model-routing.nix
      vm.nix
  profiles/
    dev-vm.nix
    plasma.nix
    iso.nix
  pkgs/
    agntctl/
    agntd/
    agnt-settings/
  crates/
    agntctl/
    agntd/
    agnt-common/
  skills/
    os/
      inspect-hardware/
      edit-nix-config/
      enable-service/
      install-app/
  .specs/
```

This structure can change as the implementation teaches us more, but the separation between modules, profiles, packages, crates, and skills should remain.

## NixOS Layer

The Nix layer owns:

- System module definitions.
- Plasma desktop defaults.
- Dev VM output.
- Future ISO output.
- Packaging for AgntOS Rust binaries.
- User and system service definitions.

The dev VM should be real NixOS, not a local root directory. For fast iteration, the VM should mount the local repository into the guest so binaries/UI can run in dev mode while release builds remain reproducible.

## `agntctl`

`agntctl` is the stable OS-control surface. All agent tool calls ultimately run `agntctl` as a subprocess.

Command families:

- `agntctl inspect`: read-only system and config inspection.
- `agntctl propose`: produce planned changes and diffs.
- `agntctl apply`: apply approved changes (with optional `--no-rebuild`).
- `agntctl audit`: show prior actions.
- `agntctl model`: list and route model profiles.
- `agntctl memory`: inspect and edit agent memory directly.

Direct Nix editing is allowed, but scoped to an AgntOS-managed config tree. The agent never touches arbitrary user Nix config.

## `agntd`

`agntd` is the LLM-powered agent daemon.

Responsibilities:

- System prompt assembly (memory snapshot + system profile + tool definitions + rules).
- LLM API client (OpenAI-compatible `/v1/chat/completions`).
- Tool call parsing and execution via `agntctl` subprocess.
- Confirmation flow for destructive operations.
- Memory management (add/replace/remove facts, capacity enforcement, security scanning).
- Session search (SQLite FTS5 for historical queries).
- Conversation turn persistence.
- Audit integration.

`agntd` connects to the LLM endpoint the user configures in `models.toml`. It does not bundle or default to any model.

## Model Routing

Model routing is task-class based, defined in a TOML file at `/etc/agntos/models.toml`.

```toml
[default]
endpoint = "http://localhost:8081/v1"
model = "qwen2.5-coder:14b"
api_key_env = "AGNTOS_API_KEY"
max_tokens = 4096
temperature = 0.7

[routing]
inspect = "default"
propose = "default"
apply = "default"
chat = "default"
memory = "default"
```

The user configures this file. `localhost:8081` is an example — each user chooses their own endpoint.

Initial task classes: `inspect`, `propose`, `apply`, `chat`, `memory`. More can be added (e.g. `vision`, `code`, `log_analysis`).

## UI Layer

Kirigami is the likely v0 GUI toolkit because AgntOS targets Plasma. The UI should eventually include:

- Agent chat.
- Approval prompts.
- Model routing settings.
- API key and endpoint setup.
- Local model management.
- Permissions and audit log.

A CLI or TUI can exist as a development interface, but it is not a separate product target in the first Plasma-only scope.

## Safety And Audit

Every OS-changing action should record:

- Requested task.
- Actor: user, agent, or system.
- Proposed config change.
- Applied files.
- Command or rebuild result.
- Rollback hint.
- Timestamp.

The audit log is JSONL at `/var/log/agntos/audit.jsonl`. Append-only. Read by `agntctl audit`.

## Extension Points

- Alternate desktop editions.
- AI Anywhere overlay.
- Background automation workspace.
- Additional model backends.
- Skills system.
- Hermes integration or fork.
- Stronger policy engine.
- GUI installer customization.
