#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."

export PRJ_ROOT="$(pwd)"

echo "==> Building agntos-cc frontend (cached npm deps)..."
nix build .#agntos-cc-frontend --print-build-logs

echo "==> Building NixOS dev VM (this may take a while on first run)..."
nix build --impure .#nixosConfigurations.agntos-dev-vm.config.system.build.vm --print-build-logs

echo "==> Starting VM (SSH: ssh -p 2222 developer@localhost, password agntos)"
echo "    Launch Control Centre from the Plasma app menu, or: DISPLAY=:0 agntos-cc"
exec ./result/bin/run-agntos-dev-vm
