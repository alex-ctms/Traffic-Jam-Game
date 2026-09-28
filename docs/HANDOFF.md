# HANDOFF

## Current Milestone
Milestone 1: tiny complete city loop. The network graph layer is built and tested;
zoning, demand, and routing are not started.

## State as of this commit
- `ops/verify.sh` is green: fmt, clippy `-D warnings`, and tests all pass.
- 14 tests pass. 7 of them are property and stress tests for `LaneGraph` that were
  written on card `t_99925683` and stranded uncommitted until they were ported.
- `tj_game` boots a Bevy 0.13 app. Note that `add_startup_system` is a Bevy 0.14+
  API; this project is on 0.13.2, so the 0.13 form `add_systems(Startup, ...)` is
  required.

## Test Status
- `tj_network` unit tests: add/query, remove/query, duplicate add, invariants
  across multiple operations, lane lookup, direction correctness, multi-segment.
- `tj_network` property and stress tests: two seeded random add/remove sequences
  asserting `check_invariants()` after every mutation, add/remove accounting,
  lane lookup consistency after removal, `Direction::opposite` involution,
  absent-road removal, and a 100k road scale case.
- No integration or benchmark suites exist yet. The scale benchmark ladder in
  `docs/SCALING.md` is specified but not implemented.

## Known Blockers
None. The build was red until commit `aa6e435` fixed the Bevy API mismatch.

## Exact Next Action
Implement the zoning layer: assign land use to road segments, then a demand model
that produces trips between homes and jobs. `tj_population` and `tj_demand` are
empty stubs.

## Next Steps
- Zoning: land-use allocation per road segment.
- Demand: trip generation between residential and job zones.
- Routing: path assignment over `LaneGraph`, which now has the invariant checks
  and stress coverage to support it.
- Headless integration test that runs N ticks and asserts population conservation.
- Implement the scale benchmark ladder so the 100k road timing is recorded
  deliberately instead of from inside a test.

## Verification
Run `./ops/verify.sh`. A non-zero exit means the work is not landable, regardless
of what any card claims about its own status.
