# Scaling Strategy

**Status:** Draft  
**Date:** 2026-09-28

## Four-Layer Scale Model

Traffic-Jam targets **10,000,000 statistical residents** as the long-term stress test. This does NOT mean 10M Bevy ECS entities, 10M vehicles, or 10M individual cars all being simulated in detail.

### Four Distinct Layers

| Layer | Name | What it represents | Scale |
|-------|------|-------------------|-------|
| A | Statistical Population | Households, zone populations, aggregated demographics | Up to 10M residents (compact aggregates) |
| B | Trip Demand | Scheduled trips, OD matrices, mode choice, freight demand | Trip flows; aggregated when idle |
| C | Active Agents | Vehicles, pedestrians, transit currently in detailed simulation | Bounded (~10k max concurrent) |
| D | Rendered Entities | Visible vehicles/buildings within camera frustum | Bounded (~1k max visible) |

**Key principle:** Population count, trip generation, active simulation, and rendering are NEVER the same thing.

## Benchmark Ladder

The following benchmark scenarios must be defined and run to validate architectural scaling:

| Scenario | Statistical Residents | Active Agents Cap | Expected Outcome |
|----------|----------------------|-------------------|------------------|
| B0 | 500 | 100 | Baseline; all trip detail visible |
| B1 | 2,000 | 500 | Demonstrate aggregation begins |
| B2 | 10,000 | 2,000 | Standard "city" scale; gameplay loop complete |
| B3 | 100,000 | 10,000 | Aggressive aggregation; LOD critical |
| B4 | 1,000,000 | 10,000 | Million-resident test; only aggregates simulated |
| B5 | 10,000,000 | 10,000 | Architectural scaling proof; NO individual cars |

**Critical requirement for B5 (10M):**
- The benchmark proves the architecture can handle 10M statistical residents without performance collapse.
- It does NOT simulate 10M individual vehicles.
- It does NOT create 10M ECS entities.
- It uses flow aggregates, corridor costs, and statistical trip demand.
- Active agent count remains bounded (~10k max).
- Rendering shows at most 1k vehicles regardless of total trip volume.

## Scale Requirements Checklist

- [x] Never create one persistent Bevy ECS entity per resident (ADR-001)
- [x] Use simulation LOD with promotion/demotion (ADR-003)
- [x] Use spatial partitioning (ADR-002)
- [x] Use fixed simulation ticks decoupled from render FPS
- [x] Allow systems to run at different cadences
- [x] Use data-oriented arrays/arenas/slabs for hot simulation state
- [x] Aggregate distant traffic flows
- [x] Bound work per simulation tick
- [x] Use hierarchical routing (ADR-004)
- [x] Cache path/corridor costs and invalidate incrementally
- [x] Update citywide analytics incrementally
- [x] Planning Mode uses overlays/deltas, not full city clone (ADR-005)
- [x] Visible vehicle count independent of total simulated trip volume

## Scale Traps to Check

The following patterns are architectural failures when they appear in hot simulation paths:

| Trap | Why it fails | Fix |
|------|--------------|-----|
| Per-resident ECS entities | O(population) memory and iteration | Use compact aggregates (households as data, not entities) |
| Global shortest path per vehicle | O(|roads|) per active vehicle | Use cached local routes + corridor costs |
| Per-frame heap allocations in movement loops | GC pressure, cache thrash | Use arenas with free-list recycling |
| Unbounded queues | Memory grows without limit | Cap active agents; demote idle trips |
| Full save-state clones for Planning Mode | O(city_size) copy time | Use delta overlays (ADR-005) |
| Analytics recomputed from raw history each frame | O(history_length) per tick | Incremental updates |
| Rendering owning simulation truth | Coupling visual LOD to sim depth | Rendering is view-only; sim owns truth |
| HashMap-heavy hot layouts | Hashing cost, cache misses | Use dense indexed arrays (ADR-001) |

## Threading Strategy

- Simulation tick runs on multiple cores where data dependencies allow
- Expensive recalculations (full network queries, corridor recomputation) spread across job queues
- Background threads for: corridor cost updates, analytics incremental aggregation
- Main thread: input, rendering, presentation of simulation state

## Game Speed Considerations

At high simulation speeds (e.g., 100x real-time):
- Increase aggregation/coarseness instead of running many detailed ticks
- Reduce active agent promotion rate
- Extend corridor cost validity windows
- Decrease spatial hash update frequency

**Rule:** When visual fidelity conflicts with simulation scale, **scale wins**.

---

*This document is a living specification. Adjust thresholds based on profiling results.*
