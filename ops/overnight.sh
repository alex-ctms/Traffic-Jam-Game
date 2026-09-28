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
export HERMES_BIN="${HERMES_BIN:-$HOME/.local/bin/hermes}"

STATE="$ROOT/ops/state"
LOG="$STATE/overnight.log"
KILLFILE="$STATE/KILL"
BOARD="${OVERNIGHT_BOARD:-traffic-jam-game}"
MAX_HOURS="${OVERNIGHT_MAX_HOURS:-8}"
TICK="${OVERNIGHT_TICK:-120}"
MAX_PER_TICK="${OVERNIGHT_MAX_PER_TICK:-2}"
BASE_REF="${OVERNIGHT_BASE_REF:-$(git rev-parse HEAD 2>/dev/null || echo HEAD)}"

mkdir -p "$STATE"

# Wave planning defaults. These MUST be set before the argument parser runs,
# otherwise the parser's values get overwritten here.
PLANNER_PROFILE="${OVERNIGHT_PLANNER:-triage}"
MAX_WAVES="${OVERNIGHT_MAX_WAVES:-3}"

usage() {
  cat <<EOF
  ops/overnight.sh [--hours N] [--tick N] [--max-per-tick N] [--board SLUG]

  --hours N          wall clock budget, default 8
  --tick N           seconds between ticks, default 120
  --max-per-tick N   workers spawned per tick, default 2
  --board SLUG       board to drive, default $BOARD
  --max-waves N      next-wave plans to generate when the board drains, default 3
  --planner PROFILE  profile that writes the next wave, default triage

  Set --max-waves 0 to stop cleanly when the board drains instead of planning more.

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
    --max-waves) MAX_WAVES="$2"; shift ;;
    --planner) PLANNER_PROFILE="$2"; shift ;;
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

# Count cards that can actually run right now: status ready AND assigned.
#
# The board keeps an unassigned EPIC container card in `ready` forever by
# design, so raw stats ready=1 never reaches 0 and a naive "drained" test never
# fires. An unassigned card consumes no worker slot and will never be claimed,
# so it must not count as runnable work.
runnable_ready() {
  # kanban list columns are: <marker> <task_id> <status> <assignee> <priority> <title>
  # The marker is a glyph (done is a check mark), so the id is not always $1.
  # Locate the id by pattern, then take the next two fields as status+assignee.
  hermes kanban list 2>/dev/null | awk '
    {
      for (i = 1; i <= NF; i++) {
        if ($i ~ /^t_[0-9a-f]{8}$/) { id = i; break }
      }
      if (!id) next
      status = $(id + 1)
      assignee = $(id + 2)
      if (status == "ready" && assignee != "(unassigned)" && assignee != "-") n++
      id = 0
    }
    END { print n + 0 }
  '
}

# A card is only real if its worktree BRANCH advanced. Two failure modes matter:
#   empty  - nothing produced at all
#   escape - the agent committed to the main checkout instead of its worktree
#            branch, which destroys isolation and can collide with other agents
# Direction matters: BASE..HEAD counts commits the worktree ADDED. The inverse
# (HEAD..BASE) counts commits it is missing and yields a false pass.
audit_one() {
  local id="$1" wt="$ROOT/.worktrees/$id"
  local branch="wt/$id" fork main_head wt_head dirty ahead
  main_head=$(git rev-parse HEAD 2>/dev/null)
   # No branch AND no directory means the card left no artifact at all.
   # A warning let fabricated "done" cards idle forever; this is a failure.
  if ! git show-ref --verify -q "refs/heads/$branch"; then
     [ -e "$wt" ] || { echo "vanished $id"; return 1; }
  fi
   # The honest baseline is the card's OWN fork point (oldest reflog entry of
   # wt/<id>), never BASE_REF. The supervisor commits its own fixes to master
   # during a run, so an old BASE_REF makes the dispatcher's empty wt/<id>
   # stamp look like the worker's genuine "commits=1". Two cards passed audit
   # exactly that way while producing nothing.
  fork=$(git reflog show --format='%H' "$branch" 2>/dev/null | tail -1)
   [ -n "$fork" ] && git cat-file -e "${fork}^{commit}" 2>/dev/null || fork="$BASE_REF"
  dirty=$(git -C "$wt" status --porcelain 2>/dev/null | grep -v '^?? ops/state' | wc -l)
   # Compare TREES, not just commit counts: allow-empty means a commit can
   # exist while every file is untouched.
  if [ -n "$fork" ] && ! git -C "$wt" diff --quiet "$fork" HEAD -- ':!ops/state' 2>/dev/null; then
    ahead=$(git -C "$wt" rev-list --count "$fork..HEAD" -- ':!ops/state' 2>/dev/null || echo 0)
    echo "ok $id commits=$ahead"
    return 0
  fi
  if [ -e "$wt" ] && [ "${dirty:-0}" -gt 0 ]; then
    echo "uncommitted $id files=$dirty"
    return 1
  fi
   # Clean and commitless. That is a failure UNLESS the branch is already an
   # ancestor of master, which means the work landed on master by another route
   # (an escape recovery, or a hand merge). Reopening those would redo finished
   # work forever.
  wt_head=$(git -C "$wt" rev-parse HEAD 2>/dev/null)
  if [ -n "$wt_head" ] && [ -n "$main_head" ] \
      && git merge-base --is-ancestor "$wt_head" "$main_head" 2>/dev/null; then
    echo "landed $id (branch is an ancestor of master)"
    return 0
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
  local dirty f stamp dir
  dirty=$(git status --porcelain 2>/dev/null | grep -v '^?? ops/state' | grep -v '^?? \.ops/')
  if [ -n "$dirty" ]; then
    log "MAIN DIRTY - an agent is writing to the main checkout, not its worktree:"
    printf '%s\n' "$dirty" | head -8 | while read -r l; do log "    $l"; done
      # Unattended rescue: snapshot EVERYTHING (tracked diff plus untracked
      # files) into ops/state/leaks, then restore the checkout. The supervisor
       # commits to master itself, so a stray worker edit risks being swept
       # into history by the next commit. Nothing is lost: the patch is kept
       # and the file list is logged so it can be merged into its worktree.
    mkdir -p "$STATE/leaks"
    local stamp dir
    stamp=$(date +%Y%m%d-%H%M%S)
    dir="$STATE/leaks/$stamp"
    mkdir -p "$dir"
    {
      git diff -- . ':(exclude)ops/state' ':(exclude).ops'
     } > "$dir/tracked.patch" 2>/dev/null
     # Copy leaked files preserving their relative paths so recovery is a
      # plain copy back, no patch-format guessing.
    local f
    while read -r f; do
      [ -f "$f" ] || continue
      mkdir -p "$dir/$(dirname "$f")"
      cp -p "$f" "$dir/$f"
    done < <(git ls-files --others --exclude-standard | grep -vE '^ops/state/|^\.ops/')
    git diff --name-only -- . ':(exclude)ops/state' ':(exclude).ops' > "$dir/modified.txt" 2>/dev/null
    git checkout -- . 2>/dev/null
    git clean -fdq -- . ':(exclude)ops/state' ':(exclude).ops' 2>/dev/null
    log "    captured leak to ${dir#$ROOT/} and restored the checkout"
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

# A done card CANNOT be reopened: block, unblock, promote, reclaim and
# request-changes all refuse 'done' (verified in kanban_db.py). The only
# sound remediation is a supersede: checkpoint whatever the failed worker
# left, mark the card with a durable audit-failure comment, and create one
# linked retry card that continues from the checkpoint. The comment doubles
# as the "already remediated" marker so the audit stops acting on it, and
# the idempotency key stops repeats from creating duplicate retries.
remediate_failed() {
  local id="$1" why="$2"
  local wt="$ROOT/.worktrees/$id"
  grep -q 'SUPERSEDED' <(hermes kanban show "$id" 2>/dev/null) \
     && { log "audit leave-alone $id (already superseded)"; return 0; }
   # 1. Checkpoint the uncommitted work onto the card's branch so a successor
    #    can inherit it instead of starting from zero.
  if [ -e "$wt" ] \
      && [ -n "$(git -C "$wt" status --porcelain 2>/dev/null | grep -v '^?? ops/state')" ]; then
    git -C "$wt" add -A >/dev/null 2>&1
    git -C "$wt" commit -q -m "wip($id): checkpoint uncommitted after audit failure" 2>/dev/null \
       && log "checkpointed WIP for $id on wt/$id"
  fi
   # 2. One retry card, only if the failed card has fewer than two retries.
  local retries title body assignee key new
  retries=$(for st in ready todo running blocked review triage; do
              hermes kanban list --status "$st" 2>/dev/null
            done | grep -c "($id retry" || true)
  if [ "${retries:-0}" -ge 2 ]; then
    log "audit STUCK $id: $why, already retried twice - needs a human"
    return 0
  fi
  title=$(hermes kanban show "$id" --json 2>/dev/null \
           | python3 -c 'import json,sys; print((json.load(sys.stdin).get("task") or {}).get("title",""))' 2>/dev/null)
  body=$(hermes kanban show "$id" --json 2>/dev/null \
          | python3 -c 'import json,sys; print((json.load(sys.stdin).get("task") or {}).get("body",""))' 2>/dev/null)
  assignee=$(hermes kanban show "$id" --json 2>/dev/null \
              | python3 -c 'import json,sys; print((json.load(sys.stdin).get("task") or {}).get("assignee") or "devlocal")' 2>/dev/null)
   [ -n "$assignee" ] || assignee="devlocal"
   [ -n "$title" ] || { log "audit FAILED to read card $id for retry - skipping"; return 0; }
  key="$BOARD:$id:retry$((retries + 1))"
  new=$(hermes kanban create "$title ($id retry $((retries + 1)))" \
         --idempotency-key "$key" --priority "$WAVE_PRIORITY" --assignee "$assignee" \
         --workspace worktree --completion-contract local-only \
         --body "$body

Supersedes $id, which the audit failed: $why
The previous worker left a checkpoint commit on branch wt/$id
(git log --oneline wt/$id). Inherit that work with git cherry-pick or by
copying the files, finish it, and commit." 2>&1 | grep -oE '\bt_[0-9a-f]{8}\b' | head -1)
  if [ -n "$new" ]; then
    hermes kanban comment "$id" "SUPERSEDED by $new: audit failed - $why" >/dev/null 2>&1
    log "SUPERSEDED $id ($why) -> $new"
  else
    hermes kanban comment "$id" "SUPERSEDED-PENDING: audit failed - $why (retry create failed)" >/dev/null 2>&1
    log "audit could not create retry for $id ($why)"
  fi
}

audit_done() {
  local id verdict rc
  while read -r id; do
     [ -z "$id" ] && continue
    verdict=$(audit_one "$id"); rc=$?
    case "$rc" in
       0) log "audit ok $id (${verdict#ok })" ;;
        # Uncommitted work is a HARD FAILURE, not a warning. A card that edited
        # files and never committed produces nothing anyone else can see, and
        # leaving it `done` means the next agent builds on a gap.
       1) remediate_failed "$id" "${verdict#uncommitted }" ;;
       2) case "$verdict" in
           vanished*|no-worktree*) remediate_failed "$id" "marked done, no branch or worktree anywhere" ;;
            *) remediate_failed "$id" "marked done but worktree is clean and commitless" ;;
         esac ;;
    esac
  done < <(done_ids)
}

# ---------------------------------------------------------------------------
# Next-wave planning.
#
# When the board drains, the run used to just stop. That wastes the rest of the
# budget: the whole point is unattended overnight progress. So on a drained
# board, ask a planning profile to write the next wave into a file, then create
# cards from it. Planning is a separate, cheap, one-shot call, not a worker slot.
#
# Two guards keep this from running away:
#   - the plan must be valid JSON in an expected shape, or nothing is created
#   - a wave counter bounds total waves for the run
# ---------------------------------------------------------------------------
WAVE=0
WAVE_LOG="$STATE/waves.jsonl"
# Integer priority. Lower dispatches first, so later waves sort after earlier
# ones while the in-flight work keeps its place.
WAVE_PRIORITY="${OVERNIGHT_WAVE_PRIORITY:-50}"

plan_prompt() {
  cat <<'PROMPT'
You are planning the next work wave for the Traffic-Jam Rust/Bevy project.
Work in the current directory. Read these before planning:
  docs/HANDOFF.md   (current state and exact next action)
  docs/ARCHITECTURE.md, docs/DECISIONS.md, docs/SCALING.md
  docs/GAME_DESIGN.md
Then inspect the source under crates/ to see what actually exists.

Plan 3 to 5 cards that make concrete progress toward a playable city loop:
generate a map, draw roads, zone homes and jobs, simulate commuting
residents, and produce measurable congestion.

Rules for every card:
  title:  one imperative line, under 90 characters.
  assignee: exactly one of macdev, architect, ui, devlocal.
            Use macdev for Rust implementation, architect for design or
            refactoring decisions, ui for rendering, devlocal for tests,
            docs, and local chores.
  body:   what to build, which files to own, and how to verify it.
          State the verification command explicitly.
  depends_on: array of indexes into this same list, 0-based. Use it only for
          real ordering constraints. Prefer parallel cards.

Output ONLY a JSON array, no prose and no code fences, shaped like:
[{"title":"...","assignee":"devlocal","body":"...","depends_on":[]}]
PROMPT
}

create_wave() {
  local plan_raw plan_file ids=() i=0 n
  WAVE=$((WAVE + 1))
  log "board drained: planning wave $WAVE of max $MAX_WAVES"

  plan_file="$STATE/wave-$WAVE.json"
  # -z is a one-shot. The planner writes the file itself; --yolo lets it use
  # file tools without prompting. Output is discarded, the file is the contract.
  hermes -p "$PLANNER_PROFILE" --yolo -z "$(plan_prompt)" >/dev/null 2>&1 \
    || { log "planner call failed; not creating a wave"; return 1; }
  if [ ! -s "$plan_file" ]; then
    # The prompt asked for JSON on stdout, not a file. Recover it from a
    # side-channel: re-ask for stdout only.
    log "planner produced no file; retrying with stdout contract"
    plan_raw=$(hermes -p "$PLANNER_PROFILE" -z \
      "$(plan_prompt)
IMPORTANT: print ONLY the JSON array to stdout. Do not write any file." 2>/dev/null)
    # Extract the first balanced JSON array with python rather than sed. A sed
    # range grabs the closing markdown fence as well and yields invalid JSON.
    printf '%s' "$plan_raw" | python3 -c '
import sys
raw = sys.stdin.read()
start = raw.find("[")
if start < 0:
    sys.exit(0)
depth = 0
for i in range(start, len(raw)):
    if raw[i] == "[":
        depth += 1
    elif raw[i] == "]":
        depth -= 1
        if depth == 0:
            sys.stdout.write(raw[start:i + 1])
            break
' > "$plan_file"
  fi
  [ -s "$plan_file" ] || { log "planner produced no usable JSON; stopping"; return 1; }

  if ! command -v python3 >/dev/null 2>&1; then
    log "python3 unavailable; cannot validate the plan, refusing to create cards"
    return 1
  fi
  n=$(python3 - "$plan_file" <<'PY' 2>/dev/null
import json,sys
try:
    raw=open(sys.argv[1]).read().strip()
    if raw.startswith("```"):
        raw=raw.split("\n",1)[1].rsplit("```",1)[0]
    data=json.loads(raw)
    assert isinstance(data,list) and data, "not a non-empty list"
    for c in data:
        assert c.get("title") and c.get("assignee") and c.get("body"), "missing field"
        assert c["assignee"] in {"macdev","architect","ui","devlocal"}, "bad assignee"
    print(len(data))
except Exception:
    print(0)
PY
)
  [ "${n:-0}" -gt 0 ] || { log "plan failed validation; stopping rather than creating bad cards"; return 1; }
  log "plan validated: $n cards"

  while read -r line; do
    [ -z "$line" ] && continue
    title=$(python3 -c 'import json,sys;print(json.loads(sys.argv[1])["title"])' "$line" 2>/dev/null)
    assignee=$(python3 -c 'import json,sys;print(json.loads(sys.argv[1])["assignee"])' "$line" 2>/dev/null)
    body=$(python3 -c 'import json,sys;print(json.loads(sys.argv[1])["body"])' "$line" 2>/dev/null)
    [ -z "$title" ] && continue
    # --priority takes an INTEGER, not a "PH2" string. Passing a label makes
    # argparse exit 2, so every card silently failed to create and the wave
    # reported zero cards. LOWER numbers dispatch first.
    local args=(--idempotency-key "$BOARD:w$WAVE:$i" --priority "$WAVE_PRIORITY"
                --assignee "$assignee" --workspace worktree
                --completion-contract local-only --body-file -)
    deps=$(python3 -c 'import json,sys;print(" ".join(json.loads(sys.argv[1]).get("depends_on",[])))' "$line" 2>/dev/null)
    for d in $deps; do
      [ "$d" -lt "$i" ] 2>/dev/null && args+=(--parent "${ids[$d]}")
    done
    create_out=$(hermes kanban create "$title" "${args[@]}" <<<"$body" 2>&1)
    new=$(printf '%s' "$create_out" | grep -oE '\bt_[0-9a-f]{8}\b' | head -1)
    if [ -n "$new" ]; then
      ids+=("$new")
      log "  created $new [$assignee] $title"
    else
      # Log the reason. A silent failure here is how a whole wave produced
      # nothing while the log said "plan validated".
      log "  FAILED to create [$assignee] $title: $(printf '%s' "$create_out" | tr '\n' ' ' | tail -c 200)"
    fi
    i=$((i + 1))
  done < <(python3 - "$plan_file" <<'PY' 2>/dev/null
import json,sys
raw=open(sys.argv[1]).read().strip()
if raw.startswith("```"):
    raw=raw.split("\n",1)[1].rsplit("```",1)[0]
for c in json.loads(raw):
    print(json.dumps(c))
PY
)

  [ "${#ids[@]}" -gt 0 ] || { log "wave $WAVE created no cards"; return 1; }
  printf '{"wave":%d,"cards":%d,"at":"%s"}\n' "$WAVE" "${#ids[@]}" "$(date -Is)" >>"$WAVE_LOG"
  log "wave $WAVE created: ${#ids[@]} cards"
  sleep 5
  return 0
}

log "=== overnight run starting ==="
log "board=$BOARD hours=$MAX_HOURS tick=${TICK}s per_tick=$MAX_PER_TICK base=$BASE_REF"
log "inhibit=idle:sleep:handle-power-key:handle-suspend-key:handle-hibernate-key:handle-lid-switch"
[ "$MAX_WAVES" -gt 0 ] && log "auto-waves enabled: max $MAX_WAVES, planner=$PLANNER_PROFILE"

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
  RUNNABLE=$(runnable_ready)
  log "tick=$TICKS triage=$TRIAGE pending=$PENDING ready=$READY runnable=$RUNNABLE running=$RUNNING blocked=$BLOCKED done=$DONE"

  if [ "$RUNNABLE" -gt 0 ]; then
    OUT="$(hermes kanban dispatch --max "$MAX_PER_TICK" 2>&1)"
    log "dispatch: $(printf '%s' "$OUT" | tr '\n' ' ' | cut -c1-240)"
  else
    log "dispatch: nothing runnable"
  fi

  audit_done
  report_escapes
  report_main_dirty || log "main checkout is dirty; investigate before the next tick"

  read -r _ _ _ RUNNING2 _ DONE2 < <(board_counts)
  RUNNABLE2=$(runnable_ready)
  if [ "${RUNNING2:-0}" -eq 0 ] && [ "${RUNNABLE2:-0}" -eq 0 ]; then
    if [ "${PENDING:-0}" -eq 0 ]; then
      # Drained with nothing gated. Rather than idle out the remaining budget,
      # plan the next wave. Only while waves remain and time is left.
      if [ "$MAX_WAVES" -gt 0 ] && [ "$WAVE" -lt "$MAX_WAVES" ] && [ "$(date +%s)" -lt "$DEADLINE" ]; then
        if create_wave; then
          log "continuing with wave $WAVE"
        else
          STOP_REASON="board complete and planner produced nothing usable: $DONE done, $BLOCKED blocked"
          break
        fi
      else
        STOP_REASON="board complete: $DONE done, $BLOCKED blocked${MAX_WAVES:+ ($WAVE/$MAX_WAVES waves used)}"
        break
      fi
    else
      STOP_REASON="stalled: $PENDING gated, $BLOCKED blocked, 0 runnable"
      break
    fi
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
