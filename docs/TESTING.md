# Testing Strategy for Traffic‑Jam

## Scope
- **Unit tests** – pure Rust functions, deterministic, no external state.
- **Property tests** – use `proptest` to assert invariants across a wide range of random inputs.
- **Integration tests** – spin up the Bevy app in head‑less mode, execute a short simulation tick sequence, and verify observable outcomes (e.g., population count remains constant, road graph stays connected).
- **Scale‑benchmark tests** – scripted cargo commands that run the simulation for a fixed number of ticks with increasing statistical‑population sizes (500, 2 000, 10 000, 100 000, 1 000 000). Metrics recorded: CPU time, max RSS, tick‑duration percentiles.

## Commands
```bash
# Unit & property tests (fast)
cargo test --workspace --lib

# Integration tests (require X11 offscreen – use `bevy_headless` flag)
cargo test --workspace --test integration

# Scale benchmarks (run on Linux, no GPU)
cargo run --release --bin benchmark -- 500     # 500 statistical residents
cargo run --release --bin benchmark -- 2000    # 2 k
cargo run --release --bin benchmark -- 10000   # 10 k
cargo run --release --bin benchmark -- 100000  # 100 k
cargo run --release --bin benchmark -- 1000000 # 1 M
```

## Test Strategy Split
| Category | Purpose | Example Command |
|----------|---------|-----------------|
| Unit | Verify pure logic, deterministic. | `cargo test --lib simulation::tick` |
| Property | Randomised invariant checks (graph connectivity, population sum). | `cargo test property::road_graph` |
| Integration | End‑to‑end sanity (no panic, deterministic given seed). | `cargo test integration::tiny_loop` |
| Scale‑benchmark | Measure performance & memory at target scales. | `cargo run --release --bin benchmark -- 100000` |

## Determinism Verification
- All non‑random simulation steps are driven by a `SimulationRng` seeded from a user‑provided u64.
- Run the same seed twice and assert identical dump of `SimulationState` after N ticks.
- Command: `cargo test deterministic::repeatability -- --nocapture`

## Save/Load & Migration Checks
- Serialize the full `SaveState` (JSON/Bincode) after a fixed number of ticks.
- Load the saved state into a fresh binary and verify:
  1. Population totals match.
  2. Road graph topology unchanged.
  3. No orphaned entities.
- Property test: random saved snapshots round‑trip unchanged.

## Concurrency Risk Checks
- Use `loom` to model multi‑threaded access to the job‑queue and shared resource pools.
- Property test ensures no data races, deadlocks, or panics under simulated contention.
- Example command: `cargo test concurrency::job_queue --features loom`.

## Proposed Property Tests (minimum three)
1. **Road graph invariants** – Adding/removing a road preserves a connected graph for all reachable zones; no duplicate edge IDs.
2. **Trip cost monotonicity** – Adding a road never increases the shortest‑path cost between any two zones (except when the new road is isolated).
3. **Population‑demand balance** – For every simulation tick, `total_residents == sum(demand_origins) == sum(demand_destinations)`; mismatches trigger a panic.

## Continuous Integration (optional)
- Hooks can be added later to run the above commands on every commit.
- For now, developers run the commands locally before committing.

---
*Generated as part of Kanban task t_4d717d8b – QA/adversarial review of early architectural failure modes.*