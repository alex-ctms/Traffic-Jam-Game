#!/usr/bin/env bash
# Overnight supervisor for a Hermes kanban board.
#
# Two jobs, in a loop:
#   1. dispatch ready cards (one worker per profile)
#   2. AUDIT every card marked done and reopen any that produced no commit
#
# The audit exists because completion signals from these models are not
# trustworthy. A real card produced a detailed, plausible summary of writing
# 5 ADRs while leaving both target files untouched. Never trust `done`.
set -uo pipefail

ROOT="$(cd "$(dirname "$(readlink -f "$0")")/.." && pwd)"
cd "$ROOT" || exit 1
export PATH="$HOME/.local/bin:$PATH"
export HERMES_BIN="$HOME/.local/bin/hermes"

STATE="$ROOT/ops/state"
LOG="$STATE/overnight.log"
KILLFILE="$STATE/KILL"
BOARD="${OVERNIGHT_BOARD:-traffic-jam-game}"
MAX_HOURS="${OVERNIGHT_MAX_HOURS:-8}"
TICK="${OVERNIGHT_TICK:-120}"
MAX_PER_TICK="${OVERNIGHT_MAX_PER_TICK:-2}"
BASE_REF="${OVERNIGHT_BASE_REF:-$(git rev-parse HEAD 2>/dev/null || echo HEAD)}"

mkdir -p "$STATE"

usage() {
  cat <<EOF
  ops/overnight.sh [--hours N] [--tick N] [--max-per-tick N] [--board SLUG]

  --hours N          wall clock budget, default 8
  --tick N           seconds between ticks, default 120
  --max-per-tick N   workers spawned per tick, default 2
  --board SLUG       board to drive, default $BOARD

  Stops automatically when the board is finished, stalled, out of time, or
  ops/kill.sh is armed.
EOF
}

# The Desktop Surfer runner used --what=idle:sleep, which does NOT cover
# suspend, lid close, or the power key. That is why it died at 21:32 when the
# laptop hibernated. Inhibit every sleep path systemd exposes.
if [ -z "${OVERNIGHT_INHIBITED:-}" ]; then
  export OVERNIGHT_INHIBITED=1
  exec systemd-inhibit \
    --what=idle:sleep:handle-power-key:handle-suspend-key:handle-hibernate-key:handle-lid-switch \
    --who="hermes overnight ($BOARD)" \
    --why="autonomous kanban run; sleep and hibernate are blocked" \
    "$0" "$@"
fi

while [ $# -gt 0 ]; do
  case "$1" in
    --hours) MAX_HOURS="$2"; shift ;;
    --tick) TICK="$2"; shift ;;
    --max-per-tick) MAX_PER_TICK="$2"; shift ;;
    --board) BOARD="$2"; shift ;;
    -h|--help) usage; exit 0 ;;
    *) echo "unknown argument: $1" >&2; usage >&2; exit 2 ;;
  esac
  shift
done

log() { printf '%s %s\n' "$(date -Is)" "$*" | tee -a "$LOG"; }

board_counts() {
  hermes kanban stats 2>/dev/null | awk '
    /^[[:space:]]*(triage|todo|scheduled|ready|running|blocked|done)[[:space:]]/ { c[$1] = $2 }
    END { printf "%d %d %d %d %d %d", c["triage"], c["todo"] + c["scheduled"], c["ready"], c["running"], c["blocked"], c["done"] }
  '
}

done_ids() {
  hermes kanban list 2>/dev/null | awk '$2 == "done" || $3 == "done" { for (i=1;i<=NF;i++) if ($i ~ /^t_[0-9a-f]{8}$/) print $i }'
}

# A card is only real if its worktree BRANCH advanced. Two failure modes matter:
#   empty  - nothing produced at all
#   escape - the agent committed to the main checkout instead of its worktree
#            branch, which destroys isolation and can collide with other agents
# Direction matters: BASE..HEAD counts commits the worktree ADDED. The inverse
# (HEAD..BASE) counts commits it is missing and yields a false pass.
audit_one() {
  local id="$1" wt="$ROOT/.worktrees/$id"
  [ -d "$wt/.git" ] || [ -f "$wt/.git" ] || { echo "no-worktree $id"; return 2; }
  local ahead dirty
  ahead=$(git -C "$wt" rev-list --count "$BASE_REF..HEAD" 2>/dev/null || echo 0)
  dirty=$(git -C "$wt" status --porcelain 2>/dev/null | grep -v '^?? ops/state' | wc -l)
  if [ "${ahead:-0}" -gt 0 ]; then
    echo "ok $id commits=$ahead"
    return 0
  fi
  if [ "${dirty:-0}" -gt 0 ]; then
    echo "uncommitted $id files=$dirty"
    return 1
  fi
  echo "empty $id"
  return 2
}

# Flag cards whose work landed on master while their worktree branch never moved.
# Not auto-reverted: the work is real and may be wanted. Surfaced loudly so a
# human decides, because a committed-to-master agent can collide with others.
# Detect an agent writing into the MAIN checkout instead of its worktree. This is
# the failure that actually happened: a running devlocal worker created
# crates/tj_core/src/id.rs in the main checkout, producing a half-written
# duplicate of a file its own worktree already had a fuller version of. A dirty
# main checkout is therefore a hard signal, not a warning, because the next
# commit from anywhere can sweep the corruption into history.
report_main_dirty() {
  local dirty
  dirty=$(git status --porcelain 2>/dev/null | grep -v '^?? ops/state' | grep -v '^?? \.ops/')
  if [ -n "$dirty" ]; then
    log "MAIN DIRTY - an agent is writing to the main checkout, not its worktree:"
    printf '%s\n' "$dirty" | head -8 | while read -r l; do log "    $l"; done
    log "    capture these files, then git checkout -- . and remove stray untracked files"
    return 1
  fi
  return 0
}

report_escapes() {
  local id ahead main_head wt_head
  main_head=$(git rev-parse HEAD 2>/dev/null)
  while read -r id; do
    [ -z "$id" ] && continue
    ahead=$(git -C "$ROOT/.worktrees/$id" rev-list --count "$BASE_REF..HEAD" 2>/dev/null || echo 0)
    [ "${ahead:-0}" -gt 0 ] && continue
    dirty=$(git -C "$ROOT/.worktrees/$id" status --porcelain 2>/dev/null | grep -v '^?? ops/state' | wc -l)
    # Only a CLEAN worktree can have escaped. A dirty one simply forgot to commit.
    [ "${dirty:-0}" -gt 0 ] && continue
    wt_head=$(git -C "$ROOT/.worktrees/$id" rev-parse HEAD 2>/dev/null)
    if [ -n "$wt_head" ] && git merge-base --is-ancestor "$wt_head" "$main_head" 2>/dev/null; then
      log "ESCAPE $id: worktree is clean and commitless but its branch is behind master (branch=$wt_head master=$main_head) - work likely committed to master"
    fi
  done < <(done_ids)
}

reopen() {
  local id="$1" why="$2"
  hermes kanban block "$id" "AUDIT FAILURE: $why" >/dev/null 2>&1
  hermes kanban unblock "$id" >/dev/null 2>&1
  log "REOPENED $id ($why)"
}

audit_done() {
  local id verdict rc
  while read -r id; do
    [ -z "$id" ] && continue
    verdict=$(audit_one "$id"); rc=$?
    case "$rc" in
      0) log "audit ok $id (${verdict#ok })" ;;
      1) log "audit WARN $id (${verdict#uncommitted }) - work exists but uncommitted" ;;
      2) case "$verdict" in
           no-worktree*) log "audit WARN $id (no worktree on disk)" ;;
           *) reopen "$id" "marked done but worktree is clean and commitless" ;;
         esac ;;
    esac
  done < <(done_ids)
}

log "=== overnight run starting ==="
log "board=$BOARD hours=$MAX_HOURS tick=${TICK}s per_tick=$MAX_PER_TICK base=$BASE_REF"
log "inhibit=idle:sleep:handle-power-key:handle-suspend-key:handle-hibernate-key:handle-lid-switch"

git rev-parse --git-dir >/dev/null 2>&1 || { log "FATAL: not a git repo"; exit 1; }
hermes kanban boards list 2>/dev/null | grep -q "$BOARD" \
  || { log "FATAL: board '$BOARD' missing"; exit 1; }
hermes kanban boards use "$BOARD" >/dev/null 2>&1 && log "board in use: $BOARD"

DEADLINE=$(( $(date +%s) + MAX_HOURS * 3600 ))
TICKS=0
STOP_REASON="unknown"

while :; do
  [ -f "$KILLFILE" ] && { STOP_REASON="kill switch armed"; break; }
  NOW=$(date +%s)
  [ "$NOW" -ge "$DEADLINE" ] && { STOP_REASON="wall clock budget of ${MAX_HOURS}h reached"; break; }

  read -r TRIAGE PENDING READY RUNNING BLOCKED DONE < <(board_counts)
  : "${TRIAGE:=0}" "${PENDING:=0}" "${READY:=0}" "${RUNNING:=0}" "${BLOCKED:=0}" "${DONE:=0}"
  log "tick=$TICKS triage=$TRIAGE pending=$PENDING ready=$READY running=$RUNNING blocked=$BLOCKED done=$DONE"

  if [ "$READY" -gt 0 ]; then
    OUT="$(hermes kanban dispatch --max "$MAX_PER_TICK" 2>&1)"
    log "dispatch: $(printf '%s' "$OUT" | tr '\n' ' ' | cut -c1-240)"
  else
    log "dispatch: nothing ready"
  fi

  audit_done
  report_escapes
  report_main_dirty || log "main checkout is dirty; investigate before the next tick"

  read -r _ _ READY2 RUNNING2 _ DONE2 < <(board_counts)
  if [ "${RUNNING2:-0}" -eq 0 ] && [ "${READY2:-0}" -eq 0 ]; then
    if [ "${PENDING:-0}" -eq 0 ]; then
      STOP_REASON="board complete: $DONE done, $BLOCKED blocked"
    else
      STOP_REASON="stalled: $PENDING gated, $BLOCKED blocked, 0 runnable"
    fi
    break
  fi

  TICKS=$((TICKS + 1))
  log "sleeping ${TICK}s, $(( (DEADLINE - $(date +%s)) / 60 )) min of budget left"
  sleep "$TICK"
done

read -r TRIAGE PENDING READY RUNNING BLOCKED DONE < <(board_counts)
log "=== finished: $STOP_REASON ==="
log "final triage=$TRIAGE pending=$PENDING ready=$READY running=$RUNNING blocked=$BLOCKED done=$DONE"

{
  echo "Hermes overnight report - board $BOARD"
  echo "generated $(date -Is)"
  echo "stop reason: $STOP_REASON"
  echo "ticks: $TICKS"
  echo "final: done=$DONE blocked=$BLOCKED running=$RUNNING pending=$PENDING ready=$READY"
  echo
  echo "AUDIT NOTE: verify every done card has a real commit before trusting it."
  echo
  hermes kanban list 2>/dev/null || true
  echo
  echo "--- last 40 log lines ---"
  tail -40 "$LOG"
} >"$STATE/morning-report.txt" 2>&1

log "report written to $STATE/morning-report.txt"
exit 0
