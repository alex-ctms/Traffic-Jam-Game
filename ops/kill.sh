#!/usr/bin/env bash
# Arm the kill switch, or clean up an overnight run.
set -uo pipefail

ROOT="$(cd "$(dirname "$(readlink -f "$0")")/.." && pwd)"
STATE="$ROOT/ops/state"
SESSION="${OVERNIGHT_TMUX_SESSION:-overnight-$(basename "$ROOT" | tr '[:upper:]' '[:lower:]')}"
mkdir -p "$STATE"

case "${1:-}" in
  --clear)
    rm -f "$STATE/KILL"
    echo "kill switch disarmed"
    exit 0
    ;;
  --status)
    pgrep -af 'ops/overnight.sh' >/dev/null 2>&1 \
      && echo "runner: running" || echo "runner: stopped"
    tmux has-session -t "$SESSION" 2>/dev/null \
      && echo "tmux: session '$SESSION' up" || echo "tmux: no '$SESSION' session"
    [ -f "$STATE/KILL" ] && echo "kill switch: ARMED" || echo "kill switch: disarmed"
    echo "log: $STATE/overnight.log"
    exit 0
    ;;
esac

: >"$STATE/KILL"
echo "kill switch armed"
pkill -f "$ROOT/ops/overnight.sh" 2>/dev/null && echo "signalled runner"
tmux kill-session -t "$SESSION" 2>/dev/null && echo "killed tmux session $SESSION"
exit 0
