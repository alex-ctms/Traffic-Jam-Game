# Decision Log

*Placeholder*

---

## ADR-001: Stable Typed ID Representation

**Status:** Accepted  
**Date:** 2026-09-28  
**Authors:** Principal Systems Architect

### Context

The simulation must support up to 10M statistical residents, but we cannot create one Bevy ECS entity per resident. We need ID representations that are:
- Stable across simulation frames
- Typed (we must know if an ID is a household, trip, active vehicle, or rendered entity)
- Memory-efficient when stored in large numbers
- Not HashMap-dependent in hot paths

Current stub code uses raw `u64` IDs everywhere, which loses type information and risks ID collisions between domains.

### Decision

Use **typed ID wrappers** backed by **dense indexed arrays** ( arenas/slabs ) for each domain:

```rust
// Core ID types in tj_core
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct HouseholdId(pub u32);  // dense index into HouseholdArena

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct TripId(pub u32);       // dense index into TripArena

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct AgentId(pub u32);      // dense index into ActiveAgentArena

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct RenderedEntityId(pub u32);  // dense index into RenderedEntityArena

// Each domain uses an Arena<T> for storage:
// - Arena<T> wraps Vec<T> with fast index-based access
// - Free slots are tracked in a free-list Vec<u32>
// - No HashMap lookups in hot paths
```

**Why dense indexed arrays over HashMaps:**
- Cache-friendly contiguous memory layout
- O(1) access via index, no hashing overhead
- Predictable iteration order for deterministic simulation
- Free-list recycling avoids allocation churn
- Type safety via newtype wrappers prevents cross-domain ID misuse

**Implications:**
- Max household count ~4M (u32::MAX), sufficient for 10M residents in households of avg 2-4 people
- When an entity is removed, its index is returned to the free-list
- Entity generation counters (epoch) are NOT used; we accept slot reuse and rely on explicit demotion/promotion flows

### Consequences

**Positive:**
- Memory layout suitable for SIMD/data-oriented iteration
- No HashMap in hot simulation loops
- Type-safe IDs prevent bugs where a HouseholdId is accidentally used as an AgentId
- Clear boundary between statistical population (HouseholdId, TripId) and active simulation (AgentId)

**Negative:**
- Must manage free-lists explicitly
- Must ensure IDs are never leaked across domain boundaries without promotion/demotion
- Debugging may be harder without global unique IDs

**Scale traps avoided:**
- No per-resident ECS entities (HouseholdId is just an index, not an Entity)
- No HashMap lookups in movement loops

---

## ADR-002: Spatial Partitioning Scheme

**Status:** Accepted  
**Date:** 2026-09-28  
**Authors:** Principal Systems Architect

### Context

The network can grow to city scale with potentially millions of road segments. Simulations must:
- Find nearby entities without scanning the full city
- Support LOD based on camera zoom/camera distance
- Support hierarchical routing (local vs. corridor vs. citywide)
- Bound work per tick regardless of city size

Current stub code has no spatial structure; `route()` just returns empty Vec.

### Decision

Use a **hierarchical grid** combining:
1. **Static spatial grid for the network:** Fixed cells (e.g., 250m x 250m) mapping road segments to cells
2. **Dynamic spatial hash for active agents:** Updated every N ticks, mapping AgentIds to cells
3. **District layer:** Coarse cells (e.g., 2km x 2km) for hierarchical routing

Structure:

```rust
// tj_network::spatial

pub struct SpatialCell {
    pub segment_ids: Vec<RoadSegmentId>,
    pub agent_ids: Vec<AgentId>,  // updated every 10 frames
    pub district_id: DistrictId,
}

pub struct SpatialGrid {
    pub cells: Vec<SpatialCell>,  // row-major: cell_idx = (y * cols) + x
    pub cell_size_m: f32,
    pub cols: u32,
    pub rows: u32,
}

pub struct District {
    pub id: DistrictId,
    pub corridor_costs: HashMap<(DistrictId, DistrictId), CorridorCost>,
}
```

**Why fixed grid over quadtree:**
- Deterministic cell assignment O(1)
- No tree rebalancing overhead
- Predictable memory layout
- Easier to reason about LOD boundaries
- Sufficient for city-scale where map boundaries are finite

**Why update agent positions every 10 frames:**
- Most vehicles move < cell_size_m in 10 frames
- amortizes spatial hash rebuild cost
- LOD decisions remain stable for short-term planning

**District layer:**
- Corridor costs between districts are cached and invalidated incrementally
- Hierarchical routing: local path within district + inter-district backbone

### Consequences

**Positive:**
- O(1) cell lookup by map coordinate
- Bound queries to local cells; no full-city scan
- District boundaries provide natural LOD boundaries
- Corridor caching avoids recomputing inter-district paths every frame

**Negative:**
- Fixed cell size may be suboptimal for heterogeneous city density
- May need multi-scale grid for extremely large cities later

**Scale traps avoided:**
- No O(|roads|) or O(|agents|) nearest-neighbor scans
- No global shortest path per vehicle

---

## ADR-003: Simulation LOD Promotion and Demotion Thresholds

**Status:** Accepted  
**Date:** 2026-09-28  
**Authors:** Principal Systems Architect

### Context

We have four layers:
- A. Statistical population (all households, all potential trips)
- B. Trip demand (scheduled trips, aggregated by origin-destination pairs)
- C. Active agents (vehicles, pedestrians currently being simulated in detail)
- D. Rendered entities (visible to camera)

We must promote data from A→B→C when needed, and demote C→B→A when detail is no longer required.

### Decision

**Promotion rules:**

| From | To | Trigger |
|------|-----|---------|
| A → B | Household → TripDemand | Household has "ready to depart" trip scheduled at current tick |
| B → C | TripDemand → ActiveAgent | Trip enters origin cell (within N cells of road network) |
| C → D | ActiveAgent → RenderedEntity | Agent's screen-space projection size > min_pixel_threshold |

**Demotion rules:**

| From | To | Trigger |
|------|-----|---------|
| D → C | RenderedEntity → ActiveAgent | N/A (rendering never owns simulation) |
| C → B | ActiveAgent → TripDemand | Trip has reached destination cell; mark as arrived |
| B → A | TripDemand → Household | Trip aggregated into household arrival stats (e.g., "trip completed at time T") |

**Thresholds:**
- **Active agent cap:** 10,000 concurrent agents max (configurable)
- When cap reached, demote oldest idle agents to B (aggregate as "delayed trips")
- **Rendering cap:** 1,000 rendered entities max (based on camera frustum and LOD)
- **Spatial proximity for B→C:** Agent promoted when trip origin is within 5 spatial cells of active road network

**Data structures:**

```rust
// tj_demand::arena
pub struct TripDemandArena {
    // Dense array; only active trips stored
    pub trips: Vec<TripDemand>,
    // Aggregated flows: (origin_cell, destination_cell) -> count
    pub flow_aggregates: HashMap<(CellIdx, CellIdx), u32>,
}

// tj_traffic::arena
pub struct ActiveAgentArena {
    pub agents: Vec<ActiveAgent>,
    pub free_list: Vec<u32>,
    pub promotion_queue: Vec<TripId>,  // from B to C
    pub demotion_queue: Vec<AgentId>, // from C to B
}
```

### Consequences

**Positive:**
- Bounded active simulation regardless of population size
- Clear ownership: statistical data is compact aggregates; only necessary detail is simulated
- Rendering does not drive simulation depth
- Flow aggregates support citywide analytics without storing per-trip history

**Negative:**
- Aggregation loses individual trip detail (intentional)
- Promoting/demoting adds indirection overhead

**Scale traps avoided:**
- No O(population) entity churn
- Bounded work per tick
- No per-frame allocations in movement loops (arenas reuse slots)

---

## ADR-004: Hierarchical Routing

**Status:** Accepted  
**Date:** 2026-09-28  
**Authors:** Principal Systems Architect

### Context

The stub `route()` function returns empty Vec. We need a routing strategy that scales to city-wide networks without running Dijkstra/A* on the full graph for every vehicle.

### Decision

Use **hierarchical routing** with three layers:

1. **Local routing (within district):** Dijkstra/A* on local road graph. Cached per (origin_cell, destination_cell) pairs within the same district.

2. **Corridor routing (between districts):** Pre-computed corridor costs between district pairs. Stored in `District::corridor_costs`. Updated incrementally when road changes occur.

3. **Citywide routing (for Planning Mode or special cases):** Full graph query only when explicitly requested (e.g., Planning Mode simulation).

**Data structures:**

```rust
// tj_routing

pub struct LocalRouteCache {
    // Key: (origin_cell, destination_cell)
    // Value: Vec<RoadSegmentId> or estimated cost if not cached
    pub cache: HashMap<(CellIdx, CellIdx), RouteCacheEntry>,
}

pub struct CorridorCost {
    pub cost: f32,
    pub updated_tick: u64,
    pub stale_threshold_ticks: u64,
}

pub struct HierarchicalRouter {
    pub local_cache: LocalRouteCache,
    pub_district_costs: HashMap<(DistrictId, DistrictId), CorridorCost>,
}

impl HierarchicalRouter {
    // Route within same district: use local cache
    pub fn route_local(&mut self, origin: CellIdx, destination: CellIdx) -> Option<Route>;
    
    // Route between districts: combine local + corridor
    pub fn route_inter_district(&mut self, origin: CellIdx, destination: CellIdx) -> Option<Route>;
    
    // Full city-wide route (expensive, use sparingly)
    pub fn route_citywide(&self, origin: CellIdx, destination: CellIdx) -> Option<Route>;
}
```

**Invalidation strategy:**
- Corridor costs marked stale if road network changed in either district since `updated_tick`
- Local route cache invalidated when segment travel costs change > 10%
- Invalidation is incremental; full recalculations are deferred to background job

### Consequences

**Positive:**
- Most routes use cached results
- Full graph traversal only for Planning Mode or rare citywide queries
- Corridor costs amortized over many trips
- Background job queue allows spreading expensive computation

**Negative:**
- Cache staleness may lead to suboptimal routes temporarily
- Corridor abstraction may hide bottlenecks within districts

**Scale traps avoided:**
- No global shortest path per vehicle (cached local routes + corridor lookup)
- No full-city graph scans during normal simulation

---

## ADR-005: Planning Mode Representation (Overlay/Delta, Not Full City Clone)

**Status:** Accepted  
**Date:** 2026-09-28  
**Authors:** Principal Systems Architect

### Context

Planning Mode requires simulating a proposed network edit without mutating the real city. A naive approach would clone the entire city state, but that is O(city_size) memory and O(city_size) copy time—unacceptable for large cities.

The orchestrator prompt explicitly states: "Planning Mode use overlays/copy-on-write/deltas rather than cloning the whole city."

### Decision

Use **delta overlays** that reference the base city:

```rust
// tj_ops::planning

pub struct PlanningOverlay {
    // Roads added in proposed state
    pub added_segments: Vec<RoadSegment>,
    
    // Roads removed in proposed state (by ID)
    pub removed_segment_ids: HashSet<RoadSegmentId>,
    
    // Modified road properties (ID -> new properties)
    pub modified_segments: HashMap<RoadSegmentId, RoadSegmentProps>,
    
    // Snapshot of current time and analytics to compare against
    pub simulation_time: SimTime,
    pub baseline_metrics: CityMetrics,
}

// View on base + overlay
pub struct SimulatedView<'a> {
    pub base_city: &'a City,
    pub overlay: Option<&'a PlanningOverlay>,
}

impl SimulatedView<'a> {
    // Segment exists if in base or added by overlay; removed if in overlay
    pub fn segment(&self, id: RoadSegmentId) -> Option<&RoadSegment>;
    
    // Base segments are mutable only in base; overlay segments are owned
    pub fn segment_mut(&mut self, id: RoadSegmentId) -> Option<&mut RoadSegment>;
}
```

**How it works:**
1. User edits network; delta is recorded in `PlanningOverlay`
2. Simulation reads from `SimulatedView` which combines base + overlay
3. Analytics are run on `SimulatedView`
4. Resulting metrics stored in `PlanningOverlay.baseline_metrics` for comparison
5. User discards overlay (nothing touched base) or approves (overlay applied to base)

**COW for trip demand:**
- Trip demand during Planning Mode simulation references base trip aggregates
- New trips created by overlay are stored in overlay-local aggregates
- No cloning of household data

### Consequences

**Positive:**
- Memory cost of Planning Mode is O(changes), not O(city_size)
- Simulated views can be layered (experimental nested planning)
- Discarding a plan is O(1): drop the overlay
- No full save-state clone; only deltas are stored

**Negative:**
- View lookups have one extra indirection (check overlay first)
- Complex queries (e.g., "count all segments") must iterate base + added segments

**Scale traps avoided:**
- No O(city_size) copy for Planning Mode
- No per-frame allocation in movement loops (arenas still used)
- Analytics compared incrementally (baseline vs. proposed)

---

*End of ADRs*

# Decision Record: Bevy Version Selection

## Context
We need to select a Bevy version for the traffic jam game that supports headless operation on Linux.

## Decision
We choose Bevy 0.13 as it's the latest stable version compatible with the project's Rust edition and provides sufficient features while being actively maintained.

## Rationale
- Version 0.13 is compatible with Rust 2021 edition
- It has good documentation and community support
- The minimal feature set allows for headless operation without heavy dependencies

## Acceptance Criteria
- `cargo run -p tj_game` successfully boots a Bevy app in headless mode
- No system libraries are required beyond standard Linux
- Headless operation meets project needs
