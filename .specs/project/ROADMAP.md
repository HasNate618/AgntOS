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

**Memory System:**
- Single system, not two (decision: no Hermes-style background extraction)
- Two core memory files: `MEMORY.md` (system facts, < 2,200 chars) and `USER.md` (preferences, < 1,375 chars)
- Loaded as frozen snapshot into every system prompt — always in context
- Agent updates memory via `memory` tool (add/replace/remove/consolidate)
- **Agent-curated, not inferred** — the agent, with in-context judgment, decides what matters. No background extraction pipeline.
- **Don't store inspectable facts** — memory is for preferences, intent, and non-derivable user context. System state (CPU, RAM, packages) is re-inspectable via `agntctl inspect`.
- Security scanning: prompt injection, credential exfiltration, invisible Unicode
- Capacity management: >80% full triggers consolidation, 100% returns error
- Session search: SQLite FTS5 for historical queries (on-demand, not in-prompt)
- `agntctl memory` subcommand for users
- End-of-session auto-consolidation: on socket close / idle, agent reviews session and updates memory
- **Provenance tracked in audit:** `prompt` and `rationale` fields in `AuditEntry` capture the "why" behind every action

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
- The agent remembers user preferences and system facts between sessions via curated memory.
- The agent can store and retrieve facts about the system.
- Every OS mutation is recorded in the audit log with prompt/rationale for provenance.
- All changes are Nix-backed and rollback-capable (both generation-level and surgical).
- The stack works end-to-end in the dev VM.
- 55 tests pass, zero warnings, 14/14 eval checks.

## Phase 1 Expansions

Completed expansions that extend the Phase 1 agent without adding full UI or model management.

### Expansion A: Pi-Inspired General Tools

Goal: 4 minimal primitives (read_file, write_file, edit_file, run_bash) replace dozens of specialized tools.

Status: Complete.

### Expansion B: Daemon Mode & Error Handling

Goal: `agntd --socket <path>` for systemd autostart, friendly rollback errors on transient VMs.

Status: Complete.

### Expansion C: Surgical Rollback & Nix Validity

Goal: audit-log surgical undo, `--persist` flag, Nix syntax validation, option-change templates.

Status: Complete. Committed in `f06e353`. 55 tests.

### Expansion D: Memory & Provenance

Goal: improve memory architecture based on operational experience.

Decisions:
- **Single memory system, not two.** Drop Hermes-style background extraction. The agent curates its own memory via the `memory` tool — no separate inference pipeline.
- **Don't store inspectable facts.** Memory is for preferences, intent, and context that can't be derived from system state (which is re-inspectable at any time).
- **Provenance over inference.** Add `prompt` and `rationale` to `AuditEntry` to capture the "why" at the source, rather than inferring it later.
- **End-of-session auto-consolidation.** On REPL exit, the agent reviews recent session turns and updates memory via LLM extraction + consolidation.

Status: Complete.

### Expansion E: Proactive Self-Healing (Watchdogs)

Goal: `agntd` monitors system health via targeted polling checks and drafts fixes.

Approach:
- Lightweight polling loop (every 5 min): `systemctl --failed`, `df -h`, `dmesg | grep -i oom`
- If a watchdog trips, agent fetches targeted logs and evaluates
- If the issue is a config error, agent drafts a `ConfigProposal`
- User receives notification: "I've drafted a fix. Review?"
- No raw journalctl firehose — LLM evaluates only targeted, relevant logs

Status: Complete.

### Expansion F: Home Manager Integration

Goal: manage user dotfiles with the same safety and rollback guarantees as system packages.

Approach:
- Add `propose set-home-option <option> <value>` template
- Files go to `/etc/agntos/home/`
- `base.nix` imports the directory via `dirImports`
- Same propose/apply/audit/undo workflow

Status: Complete.

## Phase 2: Model Management And Routing

Goal: make model configuration a core OS feature.

Deliverables:
- Model registry and routing config.
- Cloud endpoint/API key configuration.
- Local model backend integration (already OpenAI-compatible via Ollama/llama.cpp).
- Task-class routing: chat, OS planning, config editing, log analysis, coding, future vision.
- Hardware-aware model recommendations.
- Settings UI surface (deferred to Phase 3).

Exit criteria:
- User can configure at least one cloud model and one local model path.
- User can assign models by task class.

Status: Complete (CLI). UI deferred to Phase 3.

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

## Checklist Summary

### Phase 1 -- Agent OS Foundation (Complete)

- [x] NixOS flake with dev VM and plasma profile
- [x] `agntctl`: inspect, propose, apply, audit, rollback, memory
- [x] `agntd` LLM-powered agent with streaming, tool calling, confirmation gates
- [x] Core memory: bounded curated files, security scanning, consolidation
- [x] Model routing: TOML config, task-class assignment
- [x] Socket/daemon mode for systemd autostart
- [x] Pi-inspired general tools (read/write/edit/bash)
- [x] Surgical rollback and option-change templates
- [x] Provenance tracking (prompt in audit entries)
- [x] Audit search for retrieval

### Phase 1 Expansions (In Progress)

- [x] Memory optimization: teach agent to avoid storing inspectable facts
- [x] End-of-session auto-consolidation
- [x] Proactive self-healing: targeted polling checks (systemctl --failed, disk, OOM)
- [x] Home Manager integration: user dotfiles with same propose/apply/audit/undo workflow

### Phase 2 -- Model Management & Routing

- [x] Model registry (agntctl model add/remove)
- [x] Task-class routing (agntctl model set-route)
- [x] Hardware-aware recommendations (agntctl model suggest)
- [ ] Secure API key storage (deferred — env vars suffice)

### Phase 3 -- Kirigami Settings Experience

- [ ] Agent status page
- [ ] Model routing page
- [ ] Activity/audit log viewer
- [ ] Permissions and skills management

### Phase 4 -- Full Distro Packaging

- [ ] Installable ISO
- [ ] First-run setup wizard
- [ ] AgntOS branding and default Plasma theme

### Phase 5+ -- AI Anywhere, Desktop Automation

- [ ] Screen region selection and vision model routing
- [ ] Bounded desktop automation with emergency stop
- [ ] Agent-created Plasma widgets
- [ ] Skills marketplace
