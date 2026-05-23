# Legacy (frozen)

Code here is **not built** by the default flake or Cargo workspace. It is kept for reference and recoverable via git tag `pre-wedge-a-legacy` (create during `baseline-clean` if not already present).

| Path | Was | Status |
|------|-----|--------|
| `agntos-cc/` | Tauri + React + Pi Control Centre | Frozen — see `.specs/features/pi-tauri-migration/` |
| `agntos-settings/` | Kirigami/QML GUI | Frozen — see `.specs/features/kirigami-settings/` |
| `pkgs/agntos-cc/`, `pkgs/pi-coding-agent/` | Nix packages for CC stack | Frozen — not in root `flake.nix` |
| `modules/agntos-cc.nix` | NixOS module for CC | Frozen — not imported by `base.nix` |

**Current product direction:** [.specs/project/VISION.md](../.specs/project/VISION.md)

Do not extend these trees unless VISION and STATE explicitly reopen them.
