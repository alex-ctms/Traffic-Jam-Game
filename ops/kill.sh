#!/usr/bin/env bash
# Arm the kill switch, or clean up an overnight run.
set -uo pipefail

ROOT="$(cd "$(dirname "$(readlink -f "$0")")/.." && pwd)"
STATE="$ROOT/ops/state"
SESSION="${OVERNIGHT_TMUX_SESSION:-overnight-$(basename "$ROOT" | tr '[:upper:]' '[:lower:]')}"
mkdir -p "$STATE"

# The runner is started as "bash ./ops/overnight.sh", so an anchored $ROOT
# pattern never matched and every "restart" left a stale runner alive next to
# the new one. Match the script path however it was spelled, then drop pgrep
# and anything from another checkout.
runner_pids() {
  pgrep -f 'ops/overnight\.sh' 2>/dev/null | while read -r pid; do
     [ "$pid" = "$$" ] && continue
    local cmd cwd
    cmd=$(tr '\0' ' ' <"/proc/$pid/cmdline" 2>/dev/null || true)
    case "$cmd" in *pgrep*|*kill.sh*) continue ;; esac
     # systemd-inhibit execs with the same relative argv in its cmdline, but
      # it is the WRAPPER (runner has no bash/argv0 there). Count only the
      # interpreter process.
    case "$cmd" in 'systemd-inhibit '*) continue ;; esac
     # The runner is launched with a relative argv ("bash ./ops/overnight.sh")
     # from ROOT, so match on the process working directory, not argv.
    cwd=$(readlink -f "/proc/$pid/cwd" 2>/dev/null || true)
     [ "$cwd" = "$ROOT" ] && echo "$pid"
  done
}

case "${1:-}" in
  --clear)
    rm -f "$STATE/KILL"
    echo "kill switch disarmed"
    exit 0
    ;;
  --status)
    pids=$(runner_pids)
    if [ -n "$pids" ]; then
      count=$(printf '%s\n' "$pids" | grep -c .)
      echo "runner: running ($count)"
      if [ "$count" -gt 1 ]; then
        echo "  WARNING: $count runners alive, expected at most 1:"
        printf '%s\n' "$pids" | while read -r pid; do
          printf '    pid %s started %s\n' "$pid" \
            "$(ps -o lstart= -p "$pid" 2>/dev/null | sed 's/  */ /g')"
        done
      fi
    else
      echo "runner: stopped"
    fi
    tmux has-session -t "$SESSION" 2>/dev/null \
      && echo "tmux: session '$SESSION' up" || echo "tmux: no '$SESSION' session"
    [ -f "$STATE/KILL" ] && echo "kill switch: ARMED" || echo "kill switch: disarmed"
    echo "log: $STATE/overnight.log"
    exit 0
    ;;
esac

: >"$STATE/KILL"
echo "kill switch armed"
pids=$(runner_pids)
if [ -n "$pids" ]; then
  # Kill the systemd-inhibit wrapper too, otherwise it lingers as the parent
  # of a dead script and keeps blocking sleep.
  for pid in $pids; do
    ppid=$(ps -o ppid= -p "$pid" 2>/dev/null | tr -d ' ')
    pcmd=$(tr '\0' ' ' <"/proc/$ppid/cmdline" 2>/dev/null || true)
    case "$pcmd" in systemd-inhibit*) kill "$ppid" 2>/dev/null ;; esac
    kill "$pid" 2>/dev/null
  done
  sleep 3
  for pid in $pids; do
    kill -0 "$pid" 2>/dev/null && kill -9 "$pid" 2>/dev/null
  done
  echo "signalled runner(s): $(printf '%s ' $pids)"
else
  echo "no runner was alive"
fi
tmux kill-session -t "$SESSION" 2>/dev/null && echo "killed tmux session $SESSION"
exit 0
