#!/usr/bin/env bash
set -euo pipefail

SCRIPT="$(readlink -f "${BASH_SOURCE[0]}")"
ROOT="$(cd "$(dirname "$SCRIPT")/.." && pwd)"
export PRJ_ROOT="$ROOT"

SSH_PORT="${AGNTOS_SSH_PORT:-2222}"
SSH_USER="${AGNTOS_SSH_USER:-developer}"
SSH_PASS="${AGNTOS_SSH_PASS:-agntos}"
DISK="$ROOT/agntos-dev.qcow2"
VM_BIN="$ROOT/result/bin/run-agntos-dev-vm"
LOG_DIR="$ROOT/.cache"
LOG_FILE="$LOG_DIR/agntos-vm.log"
TMUX_SESSION="${AGNTOS_TMUX_SESSION:-agntos-dev}"
TMUX_CONF="/etc/agntos/dev.tmux.conf"
EVAL_GUEST="/mnt/agntos-src/.specs/features/agntos-foundation/eval-runbook.sh"

SSH_OPTS=(
  -o StrictHostKeyChecking=no
  -o UserKnownHostsFile=/dev/null
  -o ConnectTimeout=5
  -p "$SSH_PORT"
)

usage() {
  cat <<EOF
AgntOS dev VM — single entrypoint

Usage: $(basename "$0") <command> [options]

Commands:
  go          Build (if needed), start VM, attach tmux session (default)
  build       nix build dev VM (sets PRJ_ROOT=$(pwd))
  start       Start QEMU VM in background
  stop        Stop QEMU VM
  restart     stop + start (add --reset-disk to delete $DISK)
  reset-disk  Remove persistent disk (fixes stale NixOS generation)
  status      VM / SSH / mount / agntd health
  ssh [cmd]   SSH to developer@localhost:$SSH_PORT
  tmux        SSH and attach dev tmux ($TMUX_SESSION)
  eval        Run 14-check eval runbook in VM
  logs        Tail VM QEMU log ($LOG_FILE)
  agnt-logs   Tail agntd file log + journal in VM

Environment:
  PRJ_ROOT      Repo root (auto-set)
  AGNTOS_SSH_PORT   SSH port (default 2222)

Examples:
  $(basename "$0")              # full dev loop
  $(basename "$0") build && $(basename "$0") start
  $(basename "$0") ssh -- agnt system inspect
  $(basename "$0") restart --reset-disk
EOF
}

vm_pid() {
  pgrep -f "qemu.*agntos-dev\\.qcow2" 2>/dev/null | head -1 || true
}

vm_running() {
  [ -n "$(vm_pid)" ]
}

ssh_ready() {
  ssh "${SSH_OPTS[@]}" "$SSH_USER@localhost" true 2>/dev/null
}

wait_ssh() {
  local i
  for i in $(seq 1 90); do
    if ssh_ready; then
      return 0
    fi
    sleep 2
  done
  echo "error: SSH not ready on port $SSH_PORT after 180s (see $LOG_FILE)" >&2
  return 1
}

wait_mount() {
  local i
  for i in $(seq 1 60); do
    if ssh "${SSH_OPTS[@]}" "$SSH_USER@localhost" test -f /mnt/agntos-src/flake.nix 2>/dev/null; then
      return 0
    fi
    sleep 2
  done
  echo "warning: /mnt/agntos-src not mounted (build with PRJ_ROOT set; see agntos-mount logs)" >&2
  return 1
}

cmd_build() {
  echo "==> PRJ_ROOT=$PRJ_ROOT"
  echo "==> nix build dev VM (--impure for PRJ_ROOT / 9p mount)"
  (cd "$ROOT" && nix build --impure .#nixosConfigurations.agntos-dev-vm.config.system.build.vm)
}

cmd_start() {
  mkdir -p "$LOG_DIR"
  if vm_running; then
    echo "==> VM already running (pid $(vm_pid))"
    return 0
  fi
  if [ ! -x "$VM_BIN" ]; then
    echo "error: $VM_BIN missing — run: $(basename "$0") build" >&2
    exit 1
  fi
  echo "==> Starting dev VM (log: $LOG_FILE)"
  nohup "$VM_BIN" >>"$LOG_FILE" 2>&1 &
  echo "==> Waiting for SSH..."
  wait_ssh
  echo "==> SSH ready (developer@localhost:$SSH_PORT, password $SSH_PASS)"
  wait_mount || true
}

cmd_stop() {
  local pid
  pid="$(vm_pid)"
  if [ -z "$pid" ]; then
    echo "==> VM not running"
    return 0
  fi
  echo "==> Stopping VM (pid $pid)"
  kill "$pid" 2>/dev/null || kill -9 "$pid" 2>/dev/null || true
  sleep 2
}

cmd_reset_disk() {
  cmd_stop
  if [ -f "$DISK" ]; then
    echo "==> Removing $DISK"
    rm -f "$DISK"
  else
    echo "==> No disk at $DISK"
  fi
}

cmd_restart() {
  local reset=false
  if [ "${1:-}" = "--reset-disk" ]; then
    reset=true
    shift
  fi
  cmd_stop
  if $reset; then
    rm -f "$DISK"
    echo "==> Disk reset"
  fi
  cmd_start
}

cmd_status() {
  echo "PRJ_ROOT=$PRJ_ROOT"
  echo "disk: $DISK ($( [ -f "$DISK" ] && du -h "$DISK" | cut -f1 || echo missing ))"
  if vm_running; then
    echo "vm:   running (pid $(vm_pid))"
  else
    echo "vm:   stopped"
  fi
  if ssh_ready; then
    echo "ssh:  up ($SSH_USER@localhost:$SSH_PORT)"
    ssh "${SSH_OPTS[@]}" "$SSH_USER@localhost" "
      echo mount: \$(test -f /mnt/agntos-src/flake.nix && echo ok || echo missing)
      echo agntd: \$(systemctl --user is-active agntd 2>/dev/null || echo inactive)
      echo sock:  \$(test -S \"\${XDG_RUNTIME_DIR:-/run/user/\$(id -u)}/agntd.sock\" && echo ok || echo missing)
      echo log:   \$(test -f \"\${XDG_STATE_HOME:-\$HOME/.local/state}/agntos/agntd.log\" && echo ok || echo missing)
      readlink /run/current-system 2>/dev/null | sed 's|.*/||'
    " 2>/dev/null || true
  else
    echo "ssh:  down"
  fi
}

ssh_cmd() {
  if [ $# -eq 0 ]; then
    exec ssh "${SSH_OPTS[@]}" -t "$SSH_USER@localhost"
  fi
  ssh "${SSH_OPTS[@]}" "$SSH_USER@localhost" "$@"
}

cmd_ssh() {
  if [ "${1:-}" = "--" ]; then
    shift
  fi
  ssh_cmd "$@"
}

cmd_tmux() {
  if ! ssh_ready; then
    echo "error: VM not reachable — run: $(basename "$0") start" >&2
    exit 1
  fi
  exec ssh "${SSH_OPTS[@]}" -t "$SSH_USER@localhost" agntos-tmux
}

cmd_eval() {
  if ! ssh_ready; then
    echo "error: VM not reachable — run: $(basename "$0") start" >&2
    exit 1
  fi
  ssh "${SSH_OPTS[@]}" "$SSH_USER@localhost" \
    "sudo AGNTOS_CONFIG_DIR=/etc/agntos bash $EVAL_GUEST"
}

cmd_logs() {
  mkdir -p "$LOG_DIR"
  touch "$LOG_FILE"
  exec tail -f "$LOG_FILE"
}

cmd_agnt_logs() {
  if ! ssh_ready; then
    echo "error: VM not reachable — run: $(basename "$0") start" >&2
    exit 1
  fi
  exec ssh "${SSH_OPTS[@]}" -t "$SSH_USER@localhost" \
    'LOG="${XDG_STATE_HOME:-$HOME/.local/state}/agntos/agntd.log"; echo "==> $LOG"; tail -n 40 -f "$LOG" 2>/dev/null & T=$!; journalctl --user -u agntd -f; kill $T 2>/dev/null'
}

cmd_go() {
  local do_build=false
  while [ $# -gt 0 ]; do
    case "$1" in
      --build) do_build=true ;;
      --reset-disk) cmd_reset_disk ;;
      -h|--help) usage; exit 0 ;;
      *) echo "unknown option: $1" >&2; usage; exit 1 ;;
    esac
    shift
  done
  if $do_build || [ ! -x "$VM_BIN" ]; then
    cmd_build
  fi
  if ! vm_running; then
    cmd_start
  elif ! ssh_ready; then
    wait_ssh
  fi
  cmd_tmux
}

main() {
  local cmd="${1:-go}"
  shift || true
  case "$cmd" in
    go) cmd_go "$@" ;;
    build) cmd_build ;;
    start) cmd_start ;;
    stop) cmd_stop ;;
    restart) cmd_restart "$@" ;;
    reset-disk) cmd_reset_disk ;;
    status) cmd_status ;;
    ssh) cmd_ssh "$@" ;;
    tmux) cmd_tmux ;;
    eval) cmd_eval ;;
    logs) cmd_logs ;;
    agnt-logs) cmd_agnt_logs ;;
    -h|--help|help) usage ;;
    *)
      echo "unknown command: $cmd" >&2
      usage
      exit 1
      ;;
  esac
}

main "$@"
