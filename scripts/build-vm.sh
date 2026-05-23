#!/usr/bin/env bash
set -euo pipefail
exec "$(dirname "$0")/dev-vm.sh" go "$@"
