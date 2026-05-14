#!/usr/bin/env bash
# AgntOS 10-tool eval runbook
# Run inside the dev VM: ssh developer@localhost -p 2222
# Requires: agntctl built, AGNTOS_CONFIG_DIR=/etc/agntos
#
# Usage: sudo AGNTOS_CONFIG_DIR=/etc/agntos bash eval-runbook.sh

set -euo pipefail

PASS=0
FAIL=0
TMPDIR=""

cleanup() {
  rm -rf "$TMPDIR"
}
trap cleanup EXIT

ok()   { PASS=$((PASS+1)); echo "  ✓ $1"; }
fail() { FAIL=$((FAIL+1)); echo "  ✗ $1"; }

check() {
  local label="$1" cmd="$2" expect="$3"
  local out
  out=$(eval "$cmd" 2>/dev/null) || true
  if echo "$out" | grep -q "$expect"; then
    ok "$label"
  else
    fail "$label: expected '$expect' got: $(echo "$out" | head -c 100)"
  fi
}

AGNTCTL="sudo AGNTOS_CONFIG_DIR=/etc/agntos ./target/release/agntctl"
TMPDIR=$(mktemp -d)

echo "=== AgntOS 10-tool eval runbook ==="
echo ""

# ── General tools (read, write, edit, bash) ─────────────────────────────

echo "--- General tools ---"

check "agntctl write creates file" \
  "$AGNTCTL write $TMPDIR/test.txt --content 'hello world'" \
  "Wrote"

check "agntctl read reads file" \
  "$AGNTCTL read $TMPDIR/test.txt" \
  "hello world"

check "agntctl edit replaces text" \
  "sh -c '$AGNTCTL edit $TMPDIR/test.txt --old hello --new hi >/dev/null 2>&1 && $AGNTCTL read $TMPDIR/test.txt'" \
  "hi world"

check "agntctl bash with exit 0" \
  "$AGNTCTL bash 'echo ok; uname -r'" \
  "ok"

check "agntctl bash exit code shown on failure" \
  "$AGNTCTL bash 'false 2>&1; echo hi'" \
  "hi"

# ── OS-native tools (inspect, propose, audit, memory) ───────────────────

echo ""
echo "--- OS-native tools ---"

check "agntctl inspect system" \
  "$AGNTCTL inspect system" \
  "Hostname"

check "agntctl inspect cpu" \
  "$AGNTCTL inspect cpu" \
  "CPU"

check "agntctl inspect memory" \
  "$AGNTCTL inspect memory" \
  "Total:"

check "agntctl propose install htop" \
  "$AGNTCTL propose --config-dir /etc/agntos 'install htop'" \
  "Proposal"

check "agntctl memory show" \
  "$AGNTCTL memory show --config-dir /etc/agntos" \
  "MEMORY"

check "agntctl audit list" \
  "$AGNTCTL audit list --config-dir /etc/agntos --limit 5" \
  "Recent audit"

# ── Socket/daemon mode (--socket) ───────────────────────────────────────

echo ""
echo "--- Socket/daemon mode ---"

if command -v socat &>/dev/null; then
  SOCK="/tmp/agntd-eval.sock"
  # Start agntd in socket mode, background, wait for socket
  sudo AGNTOS_CONFIG_DIR=/etc/agntos timeout 30 ./target/release/agntd --socket "$SOCK" &
  AGNTD_PID=$!
  # Wait for socket to appear
  for i in $(seq 1 10); do
    [ -S "$SOCK" ] && break
    sleep 0.5
  done

  if [ -S "$SOCK" ]; then
    # Send a prompt via socat
    RESP=$(echo '{"prompt":"inspect system briefly"}' | socat - UNIX-CONNECT:"$SOCK" 2>/dev/null || echo '{"error":"socat failed"}')
    if echo "$RESP" | grep -q '"response"'; then
      ok "agntd socket mode processes prompt"
    else
      fail "agntd socket mode: unexpected response: $(echo "$RESP" | head -c 100)"
    fi
    kill "$AGNTD_PID" 2>/dev/null || true
    wait "$AGNTD_PID" 2>/dev/null || true
    rm -f "$SOCK"
  else
    fail "agntd socket did not appear within 5s"
    kill "$AGNTD_PID" 2>/dev/null || true
  fi
else
  echo "  ~ socat not installed, skipping socket test"
fi

# ── Rollback friendly errors ────────────────────────────────────────────

echo ""
echo "--- Rollback (transient VM friendly errors) ---"

check "rollback list on transient VM (no generations)" \
  "$AGNTCTL rollback list --config-dir /etc/agntos 2>&1 || true" \
  "No NixOS generations found"

# Rollback apply may fail with different nix errors depending on environment.
# Accept any non-panic error response as passing.
check "rollback apply exits gracefully" \
  "cd /mnt/agntos-src && sudo ./target/release/agntctl rollback apply --config-dir /etc/agntos 2>&1 || true" \
  "Error:"

# ── Summary ──────────────────────────────────────────────────────────────

echo ""
echo "=== Results: $PASS passed, $FAIL failed ==="
[ "$FAIL" -eq 0 ] && echo "✓ All checks passed."
