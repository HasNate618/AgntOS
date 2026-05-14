#!/usr/bin/env bash
set -e
cd "$(dirname "$0")"
export QEMU_NET_OPTS="hostfwd=tcp::2222-:22"
exec ./result/bin/run-agntos-dev-vm
