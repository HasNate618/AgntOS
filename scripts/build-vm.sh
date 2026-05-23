#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."

export PRJ_ROOT="$(pwd)"
export NIX_CFLAGS_COMPILE="${NIX_CFLAGS_COMPILE:-}"

echo "==> PRJ_ROOT=$PRJ_ROOT (required for /mnt/agntos-src mount in VM)"
echo "==> Building NixOS dev VM (this may take a while on first run)..."
nix build --impure .#nixosConfigurations.agntos-dev-vm.config.system.build.vm --print-build-logs

echo "==> Starting VM (SSH: ssh -p 2222 developer@localhost, password agntos)"
echo "    Agent: agntd via systemd user service; agntctl on PATH from /mnt/agntos-src"
exec ./result/bin/run-agntos-dev-vm
