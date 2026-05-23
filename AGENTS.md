# AgntOS — Agent Guide

## Project

**Read `.specs/project/VISION.md` first** for product direction, wedge scope, and what is parked (GUI/Pi/CC).

AI-native NixOS distro. The LLM is the nervous system, not a sidecar. 3 mutation layers:
- System: propose -> apply -> nixos-rebuild
- Memory: agent learns -> stores -> next session knows more
- Self: agent writes skills -> gains new capabilities

## Your 10 tools

| Tool | Confirm | Use for |
|---|---|---|
| `inspect` | No | CPU, memory, GPU, disk, network |
| `propose` | No | Stage a Nix config change |
| `apply` | **Yes** (human/TUI only) | Apply proposal — not an LLM tool |
| `rollback` | **Yes** (human/TUI only) | Generation rollback or surgical undo |
| `audit` | No | Search action history by prompt/path/package |
| `memory` | No | Curate MEMORY.md and USER.md |
| `read_file` | No | Read a file |
| `write_file` | No | Create or overwrite a file |
| `edit_file` | No | Replace text in a file |
| `run_bash` | No | Execute shell commands |

Chain propose -> apply without pausing (confirmation is automatic). Use `run_bash` for `ls`, `grep`, `find`, `systemctl`, `journalctl`, `dmesg` — anything without a dedicated tool. Do not ask for new tools.

## Key rules

- **Bash is king.** LLMs parse text output fine. No structured introspection APIs needed.
- **Single memory system.** Curate MEMORY.md and USER.md yourself via the `memory` tool. No background extraction. Store preferences and intent, not inspectable facts (CPU, RAM, packages — those are re-derivable).
- **Provenance.** Every `apply` records your prompt. When asked "why was X done?", use `audit(search: "X")` to retrieve it.
- **No MCP.** The agent extends itself via bash. `curl` for APIs, `jq`/`yq`/`python3` for parsing.
- **LLM proposes only.** User or `auto_apply` policy (see `/etc/agntos/settings.json`) applies. Dev VM defaults to `auto_apply = auto`.

## Spec-driven development

Feature work lives in `.specs/features/<feature>/`. When implementing, check if a spec exists first. If you modify behavior, update the relevant docs. File structure:

```
.specs/
  project/
    PROJECT.md     Vision & goals
    ROADMAP.md     Milestones + checklist
    STATE.md       Decisions, blockers, open questions
  features/
    agntos-foundation/
      spec.md      Requirements (AGF-XXX IDs)
      design.md    Architecture & data flow
      tasks.md     Atomic tasks with verification
```

## Code conventions

- **No comments** unless logic is non-obvious. Code is self-documenting.
- **Tests** in `#[cfg(test)] mod tests` next to the code.
- **Every mutation is audited.** New state-changing tools must write to `audit.jsonl`.
- **Proposals are JSON** at `/etc/agntos/proposals/<id>.json`.
- **Nix files are generated**, never hand-edited.
- **`#[serde(default)]`** on any new serialized field (AuditEntry, ConfigProposal) for backward compat.

## Adding a tool

1. Implement in `crates/agntctl/src/`
2. CLI subcommand in `crates/agntctl/src/main.rs` (clap)
3. LLM tool definition in `crates/agntd/src/llm.rs` (`tool_definitions()`)
4. Wire execution in `crates/agntd/src/agent.rs` (`execute_tool_call()`)
5. Tests -> `cargo test` -> eval runbook

## Build & test

```bash
cargo check        # Fast feedback
cargo test         # 55+ tests, must pass
cargo build --release
cargo fmt
```

VM: `nix build --impure .#nixosConfigurations.agntos-dev-vm.config.system.build.vm` then `./result/bin/run-agntos-dev-vm`. SSH on port 2222.

Eval: `bash .specs/features/agntos-foundation/eval-runbook.sh` (14 checks)

## Project layout

```
crates/
  agnt-common/     Shared types (audit, config, memory, models, wire)
  agnt/            Unified CLI (`agnt`, `agnt system …`)
  agntctl/         OS control CLI (10 tools)
  agntd/           LLM agent daemon (prompt, tools, session store)
legacy/            Frozen GUI stacks (agntos-cc, agntos-settings) — not in workspace
modules/agntos/    NixOS modules (base, desktop-plasma, agent, vm)
profiles/          dev-vm, plasma
.specs/            VISION.md, features, eval-runbook
```

## State

- **Current:** `baseline-clean` done → next `wedge-a` (`agnt` TUI, terminal dev edition)
- **Parked:** `legacy/agntos-cc`, `legacy/agntos-settings` — do not extend without VISION update
- **Core tests:** `cargo test` on workspace (agnt-common + agntctl + agntd)
- **VM eval:** `bash .specs/features/agntos-foundation/eval-runbook.sh` (14 checks)
