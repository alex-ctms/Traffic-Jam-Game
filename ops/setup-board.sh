#!/usr/bin/env bash
# Create or refresh the Hermes Kanban board for Traffic-Jam.
# Idempotent: every card carries an idempotency key, so re-running is safe.
set -euo pipefail

ROOT="$(cd "$(dirname "$(readlink -f "$0")")/.." && pwd)"
cd "$ROOT"
export PATH="$HOME/.local/bin:$PATH"
export HERMES_BIN="$HOME/.local/bin/hermes"

BOARD="traffic-jam-game"
KEY="tj"
mkdir -p .ops

RULES=$(cat <<'EOF'

## Operating rules (non-negotiable)

- NEVER mark this card blocked to ask a question. If genuinely unsure, pick the
  safest defensible option, record it with `hermes kanban comment <id> "<note>"`,
  and proceed. Block ONLY for: a product decision that materially changes game
  direction, a destructive repo action, missing credentials, or two architecture
  options with major irreversible tradeoffs (orchestrator prompt section 19).
- Budget context: read files in targeted line ranges, never dump whole files or
  vendor sources. Stop before context compaction.
- Stay strictly inside the files you were told you own. If you find a problem
  elsewhere, report it rather than expanding scope.
- Population, trip demand, active simulation and rendered entities are NEVER the
  same thing. Never create one persistent ECS entity per resident. When visual
  fidelity and simulation scale conflict, scale wins.
- One focused local git commit per card. Never push, never force-push, never
  rewrite human commits. No build artifacts or secrets in commits.
- Report back: files changed, quality-gate output, and anything left undone.
EOF
)

# mk <key> <priority> <assignee|-> <title> ; body on stdin
mk() {
  local key="$1" prio="$2" assignee="$3" title="$4"
  local body; body="$(cat)"
  local args=(--idempotency-key "$KEY:$key" --priority "$prio"
              --workspace worktree --completion-contract local-only)
  [ "$assignee" != "-" ] && args+=(--assignee "$assignee")
  local out
  out=$(hermes kanban create "$title" "${args[@]}" --body-file - <<<"$body
$RULES")
  echo "$out" | grep -oE '\bt_[0-9a-f]{8}\b' | head -1
}

hermes kanban boards create "$BOARD" \
  --name "Traffic-Jam Game" \
  --description "Rust + Bevy transportation-first city builder. Milestone 1: tiny complete city loop." \
  --default-workdir "$ROOT" --switch >/dev/null
echo "board $BOARD ready"

EPIC=$(hermes kanban create "EPIC: Milestone 1 tiny complete city loop" \
  --idempotency-key "$KEY:epic" --completion-contract local-only \
  --body-file - <<EOF | grep -oE '\bt_[0-9a-f]{8}\b' | head -1
Milestone 1 goal (orchestrator prompt section 15): generate a map, draw roads,
create intersections, zone homes and jobs, simulate commuting residents,
produce measurable congestion, inspect the bottleneck, propose a change in
Planning Mode, simulate it WITHOUT mutating the real city, approve it, and
confirm the measurable metric moved.

This is a container card, intentionally unassigned so it never consumes a
worker slot. Phases 0 and 1 cards are its children.
EOF
)
echo "EPIC=$EPIC"

QUALITY_GATE=$(mk quality-gate 98 devlocal \
  "PH0: wire cargo fmt + clippy + test into ops/verify.sh" <<'EOF'
## Goal
Create `ops/verify.sh` as the single deterministic quality gate for the whole
workspace, plus `ops/kill.sh` and `ops/probe.sh` only if trivially derivable.
Everything else on this board depends on this gate existing.

## Files you own
- `ops/verify.sh` (new)
- `.gitignore` (append only if needed)
Do not modify any crate. Do not modify Cargo.toml.

## Acceptance
- `ops/verify.sh` is executable and runs, in order:
  1. `cargo fmt --all -- --check`
  2. `cargo clippy --workspace --all-targets --all-features -- -D warnings`
  3. `cargo test --workspace`
- Exits non-zero on the FIRST failure and names the failing step.
- Exits zero and prints a clear PASS summary when all three pass.
- Accepts flags: `--quick` (fmt + check only) and `--require-tests` (fail if
  the test binary produced no tests, so an empty run cannot masquerade as pass).
- `bash -n ops/verify.sh` is clean.

## Verify
```
bash -n ops/verify.sh
ops/verify.sh
```

## Note
Do NOT add Bevy yet. Bevy is a heavy dependency with Linux system-library
requirements and it is deliberately deferred to its own card. This card only
gates the crates that already exist.
EOF
)
echo "QUALITY_GATE=$QUALITY_GATE"

SCALE_REVIEW=$(mk scale-review 96 architect \
  "PH0: review scale architecture and the four-layer LOD design" <<'EOF'
## Goal
Act as Principal Systems Architect. Review the proposed scale architecture for
the 10M-population long-term target and the four-layer model (A statistical
population, B trip demand, C active agents, D rendered entities) from
orchestrator prompt section 3. Produce concrete, reviewable decisions rather
than general advice.

## Read
- `docs/ARCHITECTURE.md`, `docs/DECISIONS.md`, `docs/GAME_DESIGN.md`
- `Traffic-Jam_Hermes_Orchestrator_Prompt.md` sections 3, 13, 14
- the existing stub crates under `crates/`

## Files you own
- `docs/DECISIONS.md` (append ADR entries)
- `docs/SCALING.md` (append the benchmark scenario ladder)
Do not write simulation code.

## Acceptance
- At least 5 ADR entries, each with Context / Decision / Consequences, covering:
  stable typed ID representation, spatial partitioning scheme, simulation LOD
  promotion and demotion thresholds, hierarchical routing, and Planning Mode
  representation (overlay/delta, NOT a full city clone).
- `docs/SCALING.md` states the benchmark ladder: 500, 2k, 10k, 100k, 1M, 10M
  statistical residents, and states explicitly that the 10M case proves
  architectural scaling, not 10M individual cars.
- Each ADR names the data structure it commits to (dense indexed arrays,
  arenas, or slabs) and why, not "a HashMap".

## Scale traps to check explicitly
Per-resident ECS entities, global shortest path per vehicle, per-frame
allocations in movement loops, unbounded queues, full save-state clones for
Planning Mode, analytics recomputed from raw history each frame, rendering
owning simulation truth, HashMap-heavy hot layouts.
EOF
)
echo "SCALE_REVIEW=$SCALE_REVIEW"

CORE_IDS=$(mk core-ids 95 devlocal \
  "PH1: stable typed IDs and fixed-tick time types in tj_core" <<'EOF'
## Goal
Implement the foundational typed ID and time primitives in `tj_core`. Every
later crate depends on these, so they must be small, total, and cheap.

## Files you own
- `crates/tj_core/src/lib.rs` and new modules inside `crates/tj_core/src/`
- `crates/tj_core/Cargo.toml` (add deps only if strictly needed)

## Acceptance
- Newtypes with a generated or macro'd `Id` per domain: Road, Lane, Intersection,
  Household, Trip, Vehicle, Zone. Use a macro so all 7 share one implementation.
- `Id(0)` is reserved as invalid and never handed out by the allocator.
- Dense contiguous allocation, no per-ID heap allocation, no `HashMap` in the
  ID path. Expose `reserve`, `alloc`, `len`, and O(1) `get(index)`.
- No panics on double-free of the same index; document the chosen behaviour.
- A fixed-tick time type: `Tick(u64)`, with `TICKS_PER_SECOND` and helpers to
  convert between ticks and seconds. Simulation tick must be decoupled from
  render FPS (orchestrator prompt section 3).
- Unit tests covering: allocation uniqueness, invalid sentinel, recycle after
  release, and tick/second round-trip.
- No new external dependencies.

## Verify
```
ops/verify.sh --require-tests
cargo test -p tj_core
```

## Constraint
This is the foundation. Prefer a small, boring, obviously-correct design over a
clever or generic one. Document the ID layout choice in code comments only if
it is not self-evident from the code.
EOF
)
echo "CORE_IDS=$CORE_IDS"

QA_RISKS=$(mk qa-risks 94 orchestrator \
  "PH0: identify the top five early architectural failure modes" <<'EOF'
## Goal
Act as QA / adversarial reviewer. Before implementation starts, find the failure
modes most likely to invalidate the architecture later. This is a review task:
you do not own implementation.

## Read
- `Traffic-Jam_Hermes_Orchestrator_Prompt.md` sections 3, 13, 14
- `docs/ARCHITECTURE.md`, `docs/DECISIONS.md`, `docs/GAME_DESIGN.md`

## Files you own
- `docs/TESTING.md` (new)
Do not modify any crate.

## Acceptance
- `docs/TESTING.md` documents the test strategy: unit / property / integration /
  scale-benchmark split, and the exact commands for each.
- A written list of the TOP FIVE early architectural failure modes, each with:
  the trigger, why it is expensive to discover late, and the cheapest test that
  would have caught it.
- Explicitly assess: determinism of the simulation (can a bug be reproduced
  from a seed?), save/load versioning and migration, and concurrency risk in
  job-queue systems.
- Propose at least 3 property tests, e.g. road add/remove preserves graph
  invariants, route cost never increases when a road is added, and population
  totals reconcile with demand totals.
EOF
)
echo "QA_RISKS=$QA_RISKS"

NETWORK=$(mk network-graph 92 devlocal \
  "PH1: road segment graph with add/remove and invariant tests" <<'EOF'
## Goal
Implement the road segment graph in `tj_network`, built on the typed IDs from
`tj_core`. First real subsystem.

## Files you own
- `crates/tj_network/src/**`
- `crates/tj_network/Cargo.toml`

## Acceptance
- `RoadGraph` with add_road, remove_road, and O(1) or O(log n) lookups by Road ID.
- Removing a road detaches its segments and does not leave dangling references;
  a segment whose endpoints no longer connect must be reported or removed, never
  silently retained.
- Every operation maintains explicit invariants. State them in code, e.g.
  "every lane's endpoints resolve to valid intersections" and
  "adjacency lists are symmetric".
- Unit tests: add, remove, remove-then-query, duplicate add, and an invariant
  checker run after every mutation.
- No O(all roads) scan inside a single-road operation.
- No unbounded growth: removals recycle IDs via the `tj_core` allocator.

## Verify
```
ops/verify.sh --require-tests
cargo test -p tj_network
```

## Scale note
This graph is the hot structure. Prefer dense indexed storage and arena/slab
allocation. A HashMap per node is an architectural review trigger (prompt
section 13).
EOF
)
echo "NETWORK=$NETWORK"

LANES=$(mk lanes 90 devlocal \
  "PH1: lanes and direction on the road graph" <<'EOF'
## Goal
Add lane and direction modelling to `tj_network`.

## Files you own
- `crates/tj_network/src/**`

## Acceptance
- A lane is a directed traversable piece of a road, with at least one lane per
  direction. Represent direction explicitly, not as a boolean flag.
- Lane connectivity derived from intersection topology, not hardcoded.
- Adding a road creates its lanes; removing it removes them and keeps counts
  consistent.
- Tests cover: lane count for a road, direction correctness, lane lookup
  O(1)/O(log n), and the road invariant checker extended to lanes.

## Verify
```
ops/verify.sh --require-tests
cargo test -p tj_network
```

## Depends on
The road graph card. If the graph API differs from what you expect, adapt to
what exists and note the mismatch in your report rather than rewriting it.
EOF
)
echo "LANES=$LANES"

INTERSECTIONS=$(mk intersections 88 devlocal \
  "PH1: intersection model and connectivity" <<'EOF'
## Goal
Implement the intersection model in `tj_network`, the node type that joins
roads and will later carry signal state.

## Files you owned
- `crates/tj_network/src/**`

## Acceptance
- An Intersection connects road endpoints and enumerates its approaches.
- Approaches carry enough information to reason about conflicting movements
  later (incoming lane, outgoing lane, turn type) WITHOUT implementing signal
  timing yet.
- Add/remove a road that meets an intersection updates approaches correctly and
  keeps the invariant checker green.
- No per-frame or per-operation full-network scan.
- Tests cover: two roads meeting, three-way, road removal cleaning up the
  approach list, and a disconnected-intersection case.

## Verify
```
ops/verify.sh --require-tests
cargo test -p tj_network
```
EOF
)
echo "INTERSECTIONS=$INTERSECTIONS"

GEOMETRY=$(mk geometry 86 devlocal \
  "PH1: basic geometric representation for roads" <<'EOF'
## Goal
Add a minimal 2D geometric representation to `tj_network` so later phases can
place roads on a map. Vector-map style only: no polygons, no building meshes.

## Files you own
- `crates/tj_network/src/**`

## Acceptance
- A simple point and segment type. Keep it small; do not adopt a geometry crate
  for two primitives unless a real need appears.
- A road segment knows its start and end points.
- Distance and intersection-of-two-segments helpers that the renderer and the
  editor can both use, with unit tests.
- Deterministic and free of floating point surprises in comparisons: document
  the epsilon policy.

## Verify
```
ops/verify.sh --require-tests
cargo test -p tj_network
```
EOF
)
echo "GEOMETRY=$GEOMETRY"

NET_TESTS=$(mk network-tests 84 devlocal \
  "PH1: property and stress tests for the network graph" <<'EOF'
## Goal
Add the integration and property test suite for `tj_network`. Quality of the
test suite is the deliverable, not new features.

## Files you own
- `crates/tj_network/tests/**` (new integration test files)
- `crates/tj_network/src/**` only if a test exposes a real bug

## Acceptance
- Property tests: add-then-remove returns the graph to an equal state;
  adding a road never increases the cost of an existing route; invariants hold
  after a long randomized sequence of add/remove operations.
- A deterministic seed is used so any failure is reproducible. Document how to
  re-run a specific failing seed.
- A stress test building 100k road segments that completes in a reasonable
  time; record the timing in `docs/SCALING.md` as the first real data point.
- If a test exposes a genuine bug, fix it minimally and say so in the report.

## Verify
```
ops/verify.sh --require-tests
cargo test -p tj_network --release
```

## Do not add
The `proptest` or `quickcheck` dependency only if you can justify it in the
report; otherwise hand-roll a seeded pseudo-random sequence to keep the
dependency count low (prompt section 5 warns against dependency soup).
EOF
)
echo "NET_TESTS=$NET_TESTS"

BEVY=$(mk bevy-boot 80 devlocal \
  "PH0: get a Bevy app booting headless on Linux in tj_game" <<'EOF'
## Goal
Make Bevy boot headless on Linux. This is the highest-risk card on the board:
Bevy is a large dependency with system-library requirements, and it is
deliberately isolated here so a failure cannot block the simulation crates.

## Files you own
- `crates/tj_game/**`
- workspace `Cargo.toml` (dependency + member changes only)
- Optionally a `docs/DECISIONS.md` ADR for the Bevy version choice

## Acceptance
- `cargo run -p tj_game` starts a Bevy App and exits cleanly, OR runs a
  headless/minimal schedule on this machine.
- Bevy feature flags are trimmed to the minimum needed; do not enable
  `bevy_render` heavy features just to make a window appear.
- If a system library is missing (ALSA, udev, X11/wayland dev packages), do NOT
  install packages yourself. Record the exact package names and the failure in
  your report and leave the card blocked with that specific blocker.
- A short ADR records the chosen Bevy version and why.

## Verify
```
cargo check -p tj_game
cargo run -p tj_game
```

## Risk note
This card may legitimately fail for environmental reasons. That is a valid
outcome. A precise blocker report is more valuable than a forced workaround.
EOF
)
echo "BEVY=$BEVY"

DOCS=$(mk docs-hygiene 70 triage \
  "PH0: complete the required docs set and test/change logs" <<'EOF'
## Goal
Bring the documentation set up to the list required by orchestrator prompt
section 2. Documentation only. Do not touch any crate.

## Files you own
- `docs/TESTING.md` (create if absent)
- `docs/CHANGELOG.md` (create if absent)
- `docs/KANBAN.md` (update to mirror the Hermes board state)
- `docs/HANDOFF.md` (create: current milestone, last completed card, active
  card, exact next action, test status, known blockers)
- `docs/GAME_DESIGN.md`, `docs/ARCHITECTURE.md`, `docs/SCALING.md`,
  `docs/DECISIONS.md` (create only if absent; do not rewrite existing content)

## Acceptance
- Every document named in prompt section 2 exists and is non-empty.
- `docs/CHANGELOG.md` records the baseline workspace commit.
- `docs/HANDOFF.md` is accurate as of today and states the exact next action.
- `docs/KANBAN.md` reflects the current Hermes board rather than contradicting it.
- Do not invent architecture decisions. If a doc would need real design input,
  create it with an explicit "TODO: requires architect input" marker.

## Do not
Do not modify any `crates/**` file. Do not rewrite the orchestrator prompt or
the design PDF.
EOF
)
echo "DOCS=$DOCS"

# --- dependency graph -------------------------------------------------
link() { hermes kanban link "$1" "$2" >/dev/null && echo "  link $(basename $1) -> $(basename $2)"; }

link "$SCALE_REVIEW" "$NETWORK"
link "$SCALE_REVIEW" "$CORE_IDS"
link "$QA_RISKS"    "$NET_TESTS"
link "$QUALITY_GATE" "$CORE_IDS"
link "$CORE_IDS"    "$NETWORK"
link "$NETWORK"     "$LANES"
link "$NETWORK"     "$INTERSECTIONS"
link "$NETWORK"     "$GEOMETRY"
link "$LANES"       "$NET_TESTS"
link "$INTERSECTIONS" "$NET_TESTS"
link "$GEOMETRY"    "$NET_TESTS"

# Cards are created as `todo`; dispatch only picks up `ready`. Promote the roots
# now. Everything downstream is promoted automatically as links clear.
hermes kanban promote "$QUALITY_GATE" "no unmet dependencies" \
  --ids "$SCALE_REVIEW" "$QA_RISKS" "$DOCS" "$BEVY" >/dev/null
echo "promoted 5 root cards to ready"

printf 'EPIC=%s\nQUALITY_GATE=%s\nSCALE_REVIEW=%s\nCORE_IDS=%s\nQA_RISKS=%s\nNETWORK=%s\nLANES=%s\nINTERSECTIONS=%s\nGEOMETRY=%s\nNET_TESTS=%s\nBEVY=%s\nDOCS=%s\n' \
  "$EPIC" "$QUALITY_GATE" "$SCALE_REVIEW" "$CORE_IDS" "$QA_RISKS" "$NETWORK" \
  "$LANES" "$INTERSECTIONS" "$GEOMETRY" "$NET_TESTS" "$BEVY" "$DOCS" \
  > .ops/ids.env

echo
echo "=== board state ==="
hermes kanban list
