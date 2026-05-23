# AgntOS Vision

**Status:** Canonical product direction (2026-05). When this document conflicts with older specs (`ROADMAP.md` Phase 3 GUI-first items, `pi-tauri-migration`, Kirigami stabilization), **VISION wins** for product and architecture decisions. Update `STATE.md` when execution phase changes.

**Audience:** Humans planning the project; agents implementing features, refactors, or reviews. Read this before touching agent UX, Nix profiles, or deprecated GUI paths.

---

## Personal goals (why this repo exists)

Honest scope — not mass adoption:

| Goal | Implication |
|------|-------------|
| **Learn NixOS** deeply (flakes, modules, generations) | Module-first; dev VM as lab |
| **Portfolio piece** that stands out | Polish TUI demo, README, 2–3 min video > ISO installer |
| **Main user is the author** | Optimize bootable VM + mount-source workflow |
| **Credible, not “everyone switch OS”** | README positions as reference distro + learning project |

**Portfolio narrative:** *Linux help without forum roulette — ask in natural language; system changes are Nix-backed, auditable, and reversible.*

**Distribution strategy:** **NixOS module first** (`nixosModules.agntos` on any NixOS); **distro/ISO later** for presentation only.

**Home Manager** integration (user dotfiles via `propose set-home-option`): **post-v1**.

**License:** **GPL-3.0-or-later** (see [License](#license)). Add `LICENSE` file; align crate metadata before publish.

---

## One-liner

**AgntOS is a NixOS distribution where a local general-purpose AI agent is the primary interface to the machine — with structured, reversible system changes when you need them.**

Subline (trust / tech): *Ask, find, fix anything — and when the OS changes, every mutation is declared, reviewed, audited, and rollbackable.*

---

## What AgntOS is

| AgntOS **is** | AgntOS **is not** |
|---------------|-------------------|
| A **NixOS-only** distro (chassis + modules + profiles) | A cross-platform agent (macOS/Windows agent is out of scope) |
| An **AI-native** OS: agent, memory, and tooling are first-class | A normal NixOS spin with Ollama installed |
| A **general local agent** (coding, research, admin, life tasks) | A sysadmin-only bot that only speaks Nix |
| A system where **OS mutations** go through `propose → apply → audit → rollback` | Imperative “run apt and hope” control |
| **Open** model endpoints (user-configured, no vendor lock-in) | A bundled LLM or cloud product |

**Chassis:** NixOS. Reproducible generations, `/etc/agntos/` as the agent-owned config tree, `nixos-rebuild` as the apply boundary.

**Soul:** The agent is the main way users interact with the computer in v1. The desktop exists to host a terminal (and later richer surfaces), not to be the product.

---

## What AgntOS should be (12-month intent)

### Brand (experience promise — B + C)

Two layers of messaging, both true:

1. **Platform (B):** *A NixOS distro built for AI everywhere — chat, search, and automation are native, not bolted on.*
2. **Experience (C):** *Linux that works the way you expect in 2026 — ask anything, find anything, fix anything.*

Nix is **proof**, not the headline: declarative state, generations, audit trail.

### AI-native pillars (identity beyond the agent)

The agent is **not** the only thing that makes AgntOS “AI-native.” These are **brand pillars**; implementation is phased:

| Pillar | User promise | v1 wedge | Later |
|--------|--------------|----------|-------|
| **Agent** | One capable local assistant, always available | **Ship** (CLI/TUI + `agntd`) | GUI surfaces optional |
| **Semantic search** | Find files/knowledge by meaning, not only path | Brand / roadmap | Indexer + tool or daemon |
| **Automation** | Scheduled and reactive workflows | Watchdog proto (`agntd`) | Cron + user jobs in `/etc/agntos/` |
| **AI anywhere** | Summon help from cursor / selection | Brand / roadmap | Global shortcut + portal + vision |

Do not implement pillars 2–4 in v1 unless explicitly rescoped. Do not market them as shipped until they are.

### Competitive position (user’s eyes)

Users compare to **Codex CLI**, **OpenClaw/Hermes**, **Cursor**, not to “NixOS with a chatbox.”

| They want | Codex / OpenClaw | AgntOS differentiator |
|-----------|------------------|------------------------|
| Strong terminal agent | Yes | Match baseline in v1 (tools, sessions, streaming) |
| MCP / plugins | Yes (Codex, Hermes) | **Roadmap** (v1: native tools + bash only) |
| Messy system control | bash everywhere | **Structured mutations** via Nix + audit |
| “What did it change?” | Opaque | **Audit JSONL** + `prompt` on each mutation |
| Undo | Manual | **`nixos-rebuild --rollback`** + surgical undo |

**Moat sentence:** *Same agent power as the best CLIs; unlike them, system changes are proposals with generations and a paper trail.*

---

## Product layers (what we maintain)

```
┌─────────────────────────────────────────────────────────────┐
│  agntos-core (always maintained)                             │
│  • flake modules: base, agent, vm                           │
│  • crates: agnt-common, agntctl, agntd                      │
│  • /etc/agntos contract (packages/, options/, services/, …)  │
│  • models.toml routing, MEMORY.md / USER.md                  │
│  • eval-runbook, tests                                       │
└─────────────────────────────────────────────────────────────┘
                              ▲
┌─────────────────────────────────────────────────────────────┐
│  agntos-edition-dev (preset profile — not “installer UI”)      │
│  • Bootable VM / future ISO                                  │
│  • Cage + Foot + tmux + autologin → `agnt` TUI               │
│  • agntd user service, sample skills, models.toml.example    │
│  • Source mount /mnt/agntos-src for iteration                │
└─────────────────────────────────────────────────────────────┘
                              ▲
┌─────────────────────────────────────────────────────────────┐
│  agntos-edition-plasma (optional / deprioritized)            │
│  • Legacy KDE profile for demos only until cut or revived    │
└─────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────┐
│  PARKED / DO NOT EXPAND without explicit rescope             │
│  • agntos-settings (Kirigami/QML)                            │
│  • agntos-cc (Tauri + React + Node Pi)                       │
│  • pi-tauri-migration, cc-gui-v2, settings-stabilization     │
└─────────────────────────────────────────────────────────────┘
```

**Preset edition ≠ graphical installer.** A preset is a **NixOS profile** (packages, services, defaults). A graphical installer (Calamares, etc.) is a later distro milestone, not wedge A.

**Omarchy analogy:** Omarchy ships an opinionated Arch+Hyprland *experience*. AgntOS dev edition ships an opinionated **agent-first terminal experience** (Foot, Cage, tmux), not KDE rice.

---

## Architecture (runtime)

### Data flow (happy path)

```
User → agnt (TUI/CLI) → agntd (Unix socket or in-process)
                              │
                              ├─ system prompt: MEMORY.md + USER.md (frozen)
                              ├─ models.toml → LLM (OpenAI-compatible)
                              └─ tool calls → agntctl (subprocess)
                                      │
                                      ├─ read/write/edit/bash → audit
                                      ├─ propose → /etc/agntos/proposals/*.json
                                      └─ apply → write .nix → nixos-rebuild → audit
```

### Components (source of truth)

| Component | Role | Keep / rewrite |
|-----------|------|----------------|
| **agntctl** | OS control CLI; all mutations and inspection | **Keep** — best-tested layer (~51 tests) |
| **agntd** | LLM loop, tools, memory, sessions, socket daemon | **Keep** — extend; add TUI front-end |
| **agnt-common** | Audit, config proposals, wire protocol types | **Keep** |
| **agnt** (TUI) | User-facing terminal app (new or `agntd` subcommand) | **Build** — wedge A deliverable |
| **agntos-cc** | Pi + Tauri GUI | **Park** — do not extend for v1 |
| **agntos-settings** | Kirigami GUI | **Park** — removed from workspace members |

### Two storage layers (system vs agent state)

**Do not put durable agent state under `/etc/agntos/`.** That tree is imported by Nix and rolls back with generations.

| Layer | Path | Rolls back with generation? | Contents |
|-------|------|----------------------------|----------|
| **System config** | `/etc/agntos/` | **Yes** (intended) | `packages/`, `options/`, `services/`, `home/` (post-v1), `proposals/*.json`, `models.toml`, `flake-info`, `settings.json` |
| **Agent state** | XDG state dir (below) | **No** | `MEMORY.md`, `USER.md`, `sessions.db` |
| **Audit log** | `/var/log/agntos/audit.jsonl` | **No** (today) | Append-only mutation history |

**Agent state directory (canonical — implement during wedge A):**

```text
$XDG_STATE_HOME/agntos/     # usually ~/.local/state/agntos/
  memory/
    MEMORY.md              # 2,200 char cap
    USER.md                # 1,375 char cap
  sessions.db              # SQLite FTS5
```

Nix module option (planned): `services.agntos.stateDir` or rely on XDG when `agntd` runs as user service. `agntctl memory` reads/writes state dir, not `/etc/agntos/memory/`.

**Rationale:** After `nixos-rebuild --rollback`, the user still remembers who they are and what they discussed; audit log still explains past applies. Only **declared system config** rewinds.

### The `/etc/agntos` contract (generation-backed)

Imported by `modules/agntos/base.nix`:

- `packages/*.nix`, `options/*.nix`, `services/*.nix`, `home/*.nix` (post-v1)
- `proposals/*.json` — staged proposals (pending apply)
- `models.toml` — model routing
- `settings.json` — global policy (e.g. `auto_apply`, default `manual`)
- `flake-info` — optional flake URI for apply

The agent must **not** silently edit `/etc/nixos/configuration.nix` for OS intent. General `read_file` / `write_file` / `bash` apply to the rest of the filesystem (audited); rollback does **not** restore deleted `$HOME` files.

### Nix integration — honest assessment

**Strong (keep investing here):**

- Path traversal guards and snapshots in `apply.rs`
- Flake-aware `nixos-rebuild test|switch`
- `nix-instantiate --parse` validation on proposed `.nix`
- Surgical rollback from audit log + generation rollback
- `agntctl option` for NixOS option lookup
- Auto-import of drop-in modules from directories

**Weak (must fix or document clearly):**

- `propose` is **keyword/template-based** (`install`, `set`, `enable`, …), not LLM-generated Nix
- Free-text proposals fall through to **`custom.nix` with `# TODO`** — misleading if marketed as “AI writes config”
- v1.1+ should either: improve templates, add LLM-in-the-loop propose with validation, or document “structured phrases only”

**Agent guidance:** Prefer explicit `propose install htop` style phrases until propose generation improves. Use `agntctl option` before unfamiliar options. Use `apply` only after user confirmation (TUI gate or interactive prompt).

---

## Agent design

### Default personality: **B — OS-aware generalist**

- Primary: help with **anything** (code, questions, files, workflows).
- Always available context: user is on **AgntOS (NixOS)**; agent **can** inspect hardware, propose system changes, read audit log.
- Do **not** open every reply with Nix — only when the task touches system state or the user asks.

System prompt rules (preserve in `agntd`):

1. Chain `propose` then offer apply when appropriate; destructive ops respect confirmation.
2. Prefer structured tools over raw bash for file ops and OS mutations.
3. Use `run_bash` for diagnostics (`systemctl`, `journalctl`, `find`, …).
4. **Do not store inspectable facts** in memory (re-derive via `inspect`).
5. Store preferences, intent, and non-derivable context in `memory`.

### Tool catalog (LLM vs human)

**The LLM does not get `apply` or `rollback` tools.** It proposes; humans or policy apply.

| LLM tool | Purpose |
|----------|---------|
| `inspect` | CPU, RAM, GPU, disk, network, system |
| `propose` | Stage Nix change (template/keyword today) |
| `audit` | Mutation log (+ search) |
| `memory` | State-dir `MEMORY.md` / `USER.md` |
| `read_file`, `write_file`, `edit_file`, `run_bash` | General reach (audited) |

| Human / policy only (`agnt system …`) | Purpose |
|-------------------------------------|---------|
| `apply` | Apply proposal + rebuild |
| `rollback` | Generation or surgical undo |

**Apply policy (global setting in `/etc/agntos/settings.json`):**

| Mode | Behavior |
|------|----------|
| `manual` (default) | After `propose`, user runs `agnt system apply <id>` or TUI Approve |
| `auto` | `agntd` applies immediately after successful propose (no LLM call) |

- Setting is **global** (one policy per machine).
- **Dev VM profile** sets `auto_apply = true` for fast iteration.
- Production / portfolio demos should show **manual** at least once.

**Trust model:** Full power via bash/write for learning and portfolio honesty. Nix generations + audit recover **system** mistakes; not `$HOME` data loss. Optional scoped restrictions are post-v1.

**MCP:** Out of v1 (roadmap). v1 extensibility = bash + read-only skills + future MCP.

### Unified CLI: `agnt`

One user-facing binary (portfolio simplicity):

```text
agnt                    # TUI chat (default)
agnt chat               # explicit
agnt daemon             # foreground agntd (debug; normal = systemd user service)
agnt system inspect     # wraps agntctl OS commands
agnt system propose|apply|rollback|audit|memory|model …
```

`agntctl` / `agntd` remain as implementation crates; `agnt` is the only command on PATH in the edition profile. Systemd still runs `agntd` under the hood.

### Memory (Hermes-aligned — path migration required)

Same limits as [Hermes memory](https://hermes-agent.nousresearch.com/docs/user-guide/features/memory), stored under **agent state dir** (not `/etc/agntos/memory/`):

| File | Limit | Content |
|------|-------|---------|
| `MEMORY.md` | 2,200 chars | Environment, conventions, lessons |
| `USER.md` | 1,375 chars | Preferences, style, identity |

- Frozen snapshot per session.
- Agent-curated via `memory` tool — no background extraction pipeline.
- Security scanning on writes.
- **Session search:** `sessions.db` in same state dir.

**Do not add** Hermes external providers (Honcho, Mem0, …) without a new decision.

### Skills (v1.1 — decision **B: read-only**)

Hermes-style [SKILL.md](https://agentskills.io/) / progressive disclosure:

- Directories: `/etc/agntos/skills/` (system) and `~/.config/agntos/skills/` (user).
- Invoke: `/skill-name` or `agnt skills list` loads content into turn (no hub, no auto `skill_manage` in v1.1).
- Ship 2–3 samples: e.g. `propose-package`, `debug-service`, `read-audit`.

**v1.1 explicitly excludes:** Skills Hub, agent-authored skills loop, security scanner for hub installs.

### Model routing

- Config: `/etc/agntos/models.toml` (TOML, task-class → profile).
- CLI: `agntctl model list|add|remove|set-route|suggest`.
- No bundled model; user supplies endpoint + `AGNTOS_API_KEY` (or per-profile env).

---

## Runtime engine decision (Rust)

**Keep Rust for `agntctl` + `agntd`.** Do not rewrite the OS layer in Python.

| Option | Verdict |
|--------|---------|
| **agntd + new TUI (ratatui or similar)** | **Default path** for wedge A |
| **Node Pi + agntos-cc** | **Park** — dual stack, 0 CC tests, incomplete tools |
| **[pi_agent_rust](https://github.com/Dicklesworthstone/pi_agent_rust)** | **Reference only** — good TUI/session/skills ideas; adopting it = third identity (“Pi on Rust”) + still need `agntctl` extension |
| **Embed Hermes** | **No** — Python monolith, wrong coupling |

**Vibecoding note:** Tests (`cargo test`, eval-runbook) are the guardrail. Prefer extending small crates over importing large agent frameworks.

---

## Wedge roadmap (execution order)

### Wedge A — v1 (current focus)

**Goal:** One credible end-to-end path: boot VM → configure model → `agnt` TUI → chat → tools → propose → confirm → apply → audit → rollback.

| Deliverable | Notes |
|-------------|-------|
| **agnt TUI** | Streaming, tool cards, approval for apply/rollback, slash commands (`/model`, `/new`, `/help`) |
| **agntd only** | Single daemon; disable CC/Pi in dev profile |
| **Dev VM preset** | Cage + Foot + tmux (see below) |
| **Docs** | README aligned with VISION; deprecate GUI-first story |
| **Park CC/settings** | Remove from dev-vm profile; archive specs |

**Exit criteria:**

- New contributor boots VM, runs `agnt`, completes one `propose` + `apply` + `audit` + `rollback` without Kirigami or Pi.
- `cargo test` green; eval-runbook 14/14 in VM.
- No requirement for MCP, ISO installer, or Plasma.

### v1.1

- Read-only **SKILL.md** + slash loader.
- **propose** quality: fewer `custom.nix` TODOs (templates or validated LLM assist).
- TUI: session list, `history` search UX.

### v2+

- MCP client (was brand roadmap C).
- Semantic file search (brand pillar).
- Cron/automation UX (brand pillar).
- AI anywhere (global summon).
- Optional `agntos-plasma` edition or ISO + graphical installer.

---

## Dev VM preset (agntos-edition-dev)

**Decision:** Terminal-first session; no Plasma as the dev default.

### Stack

| Piece | Role |
|-------|------|
| **cage** | Wayland compositor — minimal fullscreen kiosk |
| **foot** | Terminal emulator |
| **tmux** | Session multiplexer; persistent layout |
| **agntd** | `systemd --user` service, socket at runtime dir |
| **agnt** | Default command in tmux window (TUI client) |

### Intended login flow

1. Auto-login user `developer` (existing).
2. User systemd starts `agntd`.
3. Cage runs one fullscreen Foot → tmux session.
4. tmux default: window 1 = `agnt` (or `agntctl` help); optional window 2 = shell for debugging.
5. Source repo mounted at `/mnt/agntos-src` for `cargo build` / `cargo test`.

### Remove from dev-vm profile (when implementing)

- `agntos.agntos-cc.enable`
- `desktop-plasma.nix` import (or gate behind separate `agntos-plasma` configuration)
- Heavy branding packages (Bart/KDE themes) unless needed for screenshots

### Keep in dev-vm

- `agntos.rebuild.flakeUri` → `/mnt/agntos-src#agntos-dev-vm`
- SSH port 2222, shared folder, `agntos` group
- `networkmanager`, basic dev tools (`git`, `rustc` via rustup alias or nix package)

### Nix work (not done in VISION doc — for implementers)

- New module e.g. `modules/agntos/desktop-terminal.nix` (cage, foot, tmux, autologin, tmux config).
- `profiles/dev-vm.nix` imports terminal desktop instead of `desktop-plasma.nix`.
- Optional: `programs.tmux` with AgntOS-themed status line (minimal).

---

## Repository map (for agents)

```
agntos/
  .specs/project/
    VISION.md          ← this file (product truth)
    STATE.md           ← execution phase, completed work
    ROADMAP.md         ← historical milestones; defer to VISION if conflict
    PROJECT.md         ← legacy vision; superseded by VISION for direction
  crates/
    agnt-common/       ← shared types, wire protocol
    agntctl/           ← OS CLI (KEEP)
    agntd/             ← agent daemon (KEEP, extend)
    agntos-cc/         ← PARKED
    agntos-settings/   ← PARKED (not in workspace Cargo.toml)
  modules/agntos/
    base.nix           ← /etc/agntos imports
    agent.nix          ← agntd systemd user service
    vm.nix
    desktop-plasma.nix ← optional profile only
  profiles/
    dev-vm.nix         ← retarget to terminal edition
    plasma.nix         ← optional
  flake.nix            ← agntos-dev-vm, agntos-plasma
```

**Tests (workspace):** ~93 Rust tests (`agntctl` 51 + `agntd` 27 + `agnt-common` 15). Run `cargo test` before claiming done.

**Eval:** `.specs/features/agntos-foundation/eval-runbook.sh` — 14 checks, run in VM with `AGNTOS_CONFIG_DIR=/etc/agntos`.

---

## Inspirations (what to steal vs avoid)

| Project | Steal | Avoid |
|---------|-------|-------|
| **Hermes Agent** | Memory limits, SKILL.md format, session search UX, TUI slash commands, cron *ideas* | Embedding runtime, gateway, skills hub, Honcho |
| **Codex CLI** | TUI quality bar, approval modes, model switching | Replacing agntctl; cloud-only assumptions |
| **OpenClaw** | “Personal OS” positioning, proactive automation *brand* | Node stack, duplicate agent in VM |
| **pi_agent_rust** | Terminal rendering patterns, skill invocation | Adopting as core engine |
| **Omarchy** | *Preset edition* concept (coherent defaults at install) | Hyprland-specific DE scope for v1 |

---

## Anti-patterns (do not repeat)

1. **Two agents in one VM** (`agntd` + Pi/CC) — confuses users and tests.
2. **GUI before CLI** — Kirigami/Tauri while TUI incomplete.
3. **Marketing LLM-generated Nix** while `propose` is template-only.
4. **33GB repo hygiene** — keep `target/`, `node_modules` out of product path; CC frontend not in dev edition.
5. **Expanding parked crates** without rescope in VISION/STATE.
6. **MCP in v1** — explicitly deferred.

---

## Documentation hierarchy for agents

1. **`.specs/project/VISION.md`** — what & why (this file)
2. **`.specs/project/STATE.md`** — what phase we're in, what's done
3. **`AGENTS.md`** — how to build, tool catalog, conventions
4. **`README.md`** — public face; must stay aligned with VISION
5. **Feature specs** under `.specs/features/*` — valid only where they don't contradict VISION

When implementing, if a spec says “Kirigami Phase 3 current,” treat it as **historical** unless STATE reopens that track.

---

## Open questions (explicit)

| Question | Notes |
|----------|-------|
| TUI implementation | `ratatui` in new `agnt` crate vs extend `agntd` REPL |
| `propose` v1.1 | More templates vs LLM-generated Nix + validate |
| Secure API key storage | Still env-based; keyring later |
| `agntos-plasma` fate | Separate profile vs delete |
| ISO / graphical installer | Post wedge A |
| Watchdog → user-facing cron | Design `/etc/agntos/cron/` or agntctl subcommand |

---

## Decision log (interview 2026-05)

| ID | Decision |
|----|----------|
| D-01 | Product identity: **general agent** on NixOS, not sysadmin-only |
| D-02 | Platform: **NixOS only** (not cross-platform agent) |
| D-03 | Brand: **B + C** (platform + experience); agent one pillar among several |
| D-04 | Year-one wedge: **A** — agent only; other pillars brand-only |
| D-05 | Default prompt: **B** — OS-aware generalist |
| D-06 | MCP: **C** — roadmap only, not v1 |
| D-07 | Skills: **B** — read-only SKILL.md in v1.1 |
| D-08 | Runtime: **keep Rust** (`agntctl` + `agntd`); park Pi/CC |
| D-09 | Preset: **agent-first Nix profile**, not graphical installer |
| D-10 | Dev VM: **Cage + Foot + tmux**; drop Plasma/CC from dev profile |
| D-11 | Do not adopt **pi_agent_rust** as core (reference only) |
| D-12 | Rewrite scope: **shell + TUI + dev profile**; not agntctl/Nix core |
| D-13 | Audience: **author + portfolio**; module first, ISO later |
| D-14 | **apply/rollback not LLM tools**; propose only from agent |
| D-15 | **auto_apply** global setting; default manual; **dev VM = auto** |
| D-16 | Agent state in **`~/.local/state/agntos/`**; memory survives rollback |
| D-17 | Unified **`agnt`** binary on PATH |
| D-18 | **GPL-3.0-or-later** |
| D-19 | Home Manager demos: **post-v1** |
| D-20 | Endpoints: **OpenAI-compatible only** in v1 |

---

## Portfolio demo script (60 seconds)

Three prompts that match the narrative:

1. **General:** “What's in my project under ~/… and how do I run tests?”
2. **Provenance:** “Why is this service installed?” → audit search + explain prompt
3. **Nix:** “Install htop” → propose → show `.nix` → apply (manual or auto per settings) → rollback optional

## Success definition (wedge A)

A skeptical developer (or reviewer) can:

1. Build and boot `agntos-dev-vm` (Cage + Foot + tmux).
2. Set `models.toml` and API key (OpenAI-compatible endpoint).
3. Run `agnt` and hold a general conversation (not Nix-specific).
4. Ask to install a package → get a real proposal under `packages/*.nix` (correct phrasing).
5. Apply (manual or auto per settings) → survive rebuild → audit entry retains prompt.
6. Roll back generation → **memory and chat history still present** in state dir.
7. Do all of the above **without** Plasma, Tauri, or Node Pi.

Author success: you can explain flakes, modules, generations, and the two storage layers in a blog paragraph.

That is AgntOS v1.

---

## License

**GPL-3.0-or-later** for the project.

- Fits FOSS / no telemetry / GNU alignment.
- Fine for a personal portfolio (many employers accept GPL for side projects).
- **Not** ideal if you later want others to embed AgntOS in proprietary products without sharing — unlikely given current goals.

Before publishing: add root `LICENSE`, set `license` in workspace `Cargo.toml`, remove MIT-only claims in docs if any.
