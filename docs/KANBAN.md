# Traffic-Jam Kanban

## Backlog
- [ ] TJ-010 Phase 0 - Repository Foundation
  - Owner: orchestrator/qwen3.5:27b
  - Milestone: Phase 0
  - Goal: Initialize Rust workspace, Bevy app boot, logging/config, test harness, benchmark harness, docs structure, CI‑friendly commands
  - Acceptance:
    - Cargo workspace builds
    - Bevy app starts (window can be hidden)
    - Basic tests pass (placeholder)
    - Docs folder exists with required markdown files
  - Tests: `cargo test --workspace`
  - Files: `Cargo.toml`, `crates/*/src/lib.rs`, `docs/*`
- [ ] TJ-011 Phase 1 - Road Network Skeleton
  - Owner: backend/devstral-2:123b
  - Milestone: Phase 1
  - Goal: Stable typed IDs, road segment graph, lanes, direction, intersections, basic geometry, add/remove road tests
  - Acceptance:
    - Network structs compile
    - Unit tests for add/remove roads succeed
  - Tests: `cargo test -p tj_network`
  - Files: `crates/tj_network/src/lib.rs`, `crates/tj_network/tests/*.rs`
- [ ] TJ-001 Setup Rust workspace and basic crates
- [ ] TJ-002 Create initial documentation (GDD, Architecture, Scaling)
- [ ] TJ-003 Define core types and config (tj_core)
- [ ] TJ-004 Implement network graph structures (tj_network)

## Ready
- [ ] TJ-005 Implement placeholder modules for population, demand, routing, traffic, city, ops, analytics, save, game
- [ ] TJ-006 Add initial Cargo.toml entries for all crates

## In Progress
- [ ] TJ-007 Write initial Kanban file and add to repo

## Review

## Blocked

## Done
- [x] Created workspace and Cargo.toml
- [x] Added placeholder source files for each crate
- [x] Added documentation files (GAME_DESIGN.md, ARCHITECTURE.md, SCALING.md, DECISIONS.md)
- [x] Added initial Kanban template
- [x] Successfully built entire workspace
