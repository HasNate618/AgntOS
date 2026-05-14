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

## Phase 1: Agent OS Foundation

Goal: boot into AgntOS and interact with a Hermes-like OS-aware agent.

Deliverables:
- `agntd` daemon skeleton.
- `agntctl` CLI skeleton.
- NixOS service modules.
- Structured OS tool interface: inspect, propose, apply, audit, rollback.
- First skills: inspect hardware, inspect services/logs, propose Nix config change.
- Activity log format.
- Approval flow concept.

Exit criteria:
- User can ask the agent about system state.
- User can request a simple OS change.
- `agntctl` can make or stage a Nix config edit.
- Changes are auditable.

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