# AgntOS Roadmap

Sequenced milestones. Specs preserve flexibility — the roadmap defines order and constraints, not final implementation details.

## Phase 0: Project Foundation

Goal: create a reproducible project foundation.

Deliverables:
- Git repository initialized.
- Spec-driven project memory under `.specs/`.
- Nix flake scaffold with module/profile structure.
- Plasma-only NixOS profile.
- Development VM output.
- Rust workspace stubs.

Exit criteria:
- Developer can build a VM from the repo.
- VM uses AgntOS NixOS modules rather than manual post-install setup.

Status: Complete.

## Phase 1: Agent OS Foundation

Goal: boot into AgntOS and interact with an LLM-powered agent that has persistent memory and safe OS tool access.

### Phase 1 Architecture

```
User ↔ agntd (llm-powered agent)
             ↕  tools: inspect, propose, apply, audit, memory
        agntctl (os control layer)
             ↕  writes Nix config
        /etc/agntos/ (Nix config tree)
             ↕  nixos-rebuild
        NixOS system
             ↑
        Agent memory (MEMORY.md + USER.md, always in context)
```

### Deliverables

**Model Routing Config:**
- TOML config at `/etc/agntos/models.toml`
- Model profiles: endpoint, model name, API key (from env), max_tokens, temperature
- Task-class routing: inspect / propose / apply / chat / memory each map to a profile
- `agntctl model list` and `agntctl model route <task>` subcommands
- No default endpoint — user must configure their own (localhost:8081 is an example, not built-in)

**Hermes-Style Memory System:**
- Two core memory files: `MEMORY.md` (system facts, < 2,200 chars) and `USER.md` (preferences, < 1,375 chars)
- Loaded as frozen snapshot into every system prompt — always in context
- Agent updates memory via `memory` tool (add/replace/remove)
- Security scanning: prompt injection, credential exfiltration, invisible Unicode
- Capacity management: >80% full triggers consolidation, 100% returns error
- Session search: SQLite FTS5 for historical queries (on-demand, not in-prompt)
- `agntctl memory` subcommand for users

**LLM-Powered Agent (agntd rewrite):**
- Replace keyword-matching REPL with OpenAI-compatible `/v1/chat/completions` integration
- Tool definitions: inspect, propose, apply, audit, memory as typed functions
- System prompt built from: memory snapshot + system profile + tool definitions + rules
- Stream responses if the LLM supports it
- Confirmation flow for `apply` and destructive operations
- Session persistence: save conversation turns to session store

**Tool Definitions (exposed to LLM):**

| Tool | Purpose | Safety |
|---|---|---|
| `inspect(target)` | Read system state | No confirmation needed |
| `propose(description)` | Stage a Nix config change | No confirmation needed |
| `apply(proposal_id)` | Apply staged change | Requires user confirmation |
| `audit(action, ...)` | View change history | No confirmation needed |
| `memory(action, ...)` | Update persistent memory | No confirmation needed |

### Exit Criteria

- User can configure model endpoints in `/etc/agntos/models.toml` (no hardcoded defaults).
- The agent can inspect the system, propose a change, get confirmation, and apply it.
- The agent remembers system state between sessions (e.g. "what GPU did you say I have?").
- The agent can store and retrieve facts about the system.
- Every OS mutation is recorded in the audit log.
- All changes are Nix-backed and rollback-capable.
- The stack works end-to-end in the dev VM.

## Phase 2: Model Management And Routing

Goal: make model configuration a core OS feature.

Deliverables:
- Model registry and routing config.
- Cloud endpoint/API key configuration.
- Local model backend integration (first backend TBD).
- Task-class routing: chat, OS planning, config editing, log analysis, coding, future vision.
- Hardware-aware model recommendations.
- Settings UI surface.

Exit criteria:
- User can configure at least one cloud model and one local model path.
- User can assign models by task class.

## Phase 3: Kirigami Settings Experience

Goal: polished interface for agent configuration.

Deliverables:
- Agent status page.
- Model routing page.
- Local model management page.
- Permissions page.
- Skills page.
- Activity/audit log viewer.

Exit criteria:
- Normal users can configure AgntOS without editing files.
- Power users can inspect where settings are stored.

## Phase 4: Full Distro Packaging

Goal: package AgntOS as a real installable distro.

Deliverables:
- NixOS ISO output.
- Installer customization.
- AgntOS branding.
- Default Plasma theme/config.
- First-run setup wizard.
- Release build documentation.

Exit criteria:
- User can boot the ISO in a VM.
- User can install AgntOS from the ISO.
- Installed system includes AgntOS agent services and settings.

## Phase 5: AI Anywhere

Goal: let users ask the agent about selected screen content.

Deliverables:
- Global shortcut.
- Screen region selection.
- Screenshot/context capture via XDG portals.
- OCR integration.
- Vision model routing.
- Overlay chat near selected region.

Exit criteria:
- User can select screen content and ask the agent about it.
- Feature respects user permissions and desktop constraints.

## Phase 6: Controlled Desktop Automation

Goal: agent performs bounded desktop actions.

Deliverables:
- Controlled automation mode.
- Visible action log.
- Emergency stop.
- Separate workspace/nested compositor/VM exploration.
- Input automation policy.

Exit criteria:
- User can approve bounded automation tasks.
- Agent cannot silently take over the live session.

## Long-Term Ideas

- Self-improving skills inspired by Hermes.
- Background worker desktops.
- Agent-created Plasma widgets.
- Skills marketplace.
- Local-first privacy mode.
- Developer edition.
- Hyprland Lab edition.
- Fleet/homelab management.
- Cross-device agent memory.
