# Propose package install

When the user wants a package installed:

1. Run `inspect` if you are unsure the package name or whether it is already present.
2. Call `propose` with a clear description like `install <package>`.
3. Do not call `apply` — the user or auto_apply policy applies the proposal.
4. After proposing, mention the proposal id and what will change.

Prefer Nix package names as in nixpkgs (e.g. `htop`, `ripgrep`).
