# Roadmap

## Phase 1 — Agent OS Foundation (Complete)

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

## Phase 1 Expansions (In Progress)

- [ ] Memory optimization: teach agent to avoid storing inspectable facts
- [ ] End-of-session auto-consolidation
- [ ] Proactive self-healing: targeted polling checks (`systemctl --failed`, disk, OOM)
- [ ] Home Manager integration: user dotfiles with same propose/apply/audit/undo workflow

## Phase 2 — Model Management & Routing

- [ ] Model registry (`agntctl model add/remove`)
- [ ] Secure API key storage
- [ ] Local model backend integration (Ollama, llama.cpp)
- [ ] Hardware-aware recommendations
- [ ] Task-class routing UI

## Phase 3 — Kirigami Settings Experience

- [ ] Agent status page
- [ ] Model routing page
- [ ] Activity/audit log viewer
- [ ] Permissions and skills management

## Phase 4 — Full Distro Packaging

- [ ] Installable ISO
- [ ] First-run setup wizard
- [ ] AgntOS branding and default Plasma theme

## Phase 5+ — AI Anywhere, Desktop Automation

- [ ] Screen region selection and vision model routing
- [ ] Bounded desktop automation with emergency stop
- [ ] Agent-created Plasma widgets
- [ ] Skills marketplace
