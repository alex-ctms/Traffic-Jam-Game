// Network graph structures

use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashMap, HashSet};

#[derive(Clone, Copy, Debug)]
struct PathQueueEntry {
    cost: f64,
    lane_id: u64,
}

impl PartialEq for PathQueueEntry {
    fn eq(&self, other: &Self) -> bool {
        self.cost == other.cost && self.lane_id == other.lane_id
    }
}

impl Eq for PathQueueEntry {}

impl PartialOrd for PathQueueEntry {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for PathQueueEntry {
    fn cmp(&self, other: &Self) -> Ordering {
        // BinaryHeap is a max-heap; reverse the cost ordering for Dijkstra.
        other
            .cost
            .total_cmp(&self.cost)
            .then_with(|| other.lane_id.cmp(&self.lane_id))
    }
}

pub mod planning;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Direction {
    Forward,
    Backward,
}

/// Land-use designation for the frontage of a road segment.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ZoneType {
    Residential,
    Commercial,
    Office,
    Industrial,
    Farm,
}

/// Zoning designation stored on a single road segment's frontage.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ZoneMetadata {
    pub zone_type: ZoneType,
    /// Optional player-set density cap from 0.0 through 1.0; `None` lets the market decide.
    pub max_density: Option<f32>,
}

impl ZoneMetadata {
    /// Create metadata for `zone_type` with no density cap.
    pub fn new(zone_type: ZoneType) -> Self {
        Self {
            zone_type,
            max_density: None,
        }
    }
}

/// One of the two frontage sides of a physical road segment.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FrontageSide {
    Left,
    Right,
}

impl Direction {
    pub fn opposite(&self) -> Self {
        match self {
            Direction::Forward => Direction::Backward,
            Direction::Backward => Direction::Forward,
        }
    }
}

/// Lane represents a directed traversable piece of road
#[derive(Debug, Clone, PartialEq)]
pub struct Lane {
    pub id: u64,
    pub road_id: u32,
    pub direction: Direction,
    pub start_id: u32,
    pub end_id: u32,
    pub length: f32,
}

impl Lane {
    /// Create a new lane
    pub fn new(
        id: u64,
        road_id: u32,
        direction: Direction,
        start_id: u32,
        end_id: u32,
        length: f32,
    ) -> Self {
        Self {
            id,
            road_id,
            direction,
            start_id,
            end_id,
            length,
        }
    }
}

/// Physical road segment between two intersections.
#[derive(Debug, Clone, PartialEq)]
pub struct LaneSegment {
    pub id: u64,
    pub road_id: u32,
    pub start_id: u32,
    pub end_id: u32,
    pub length: f32,
    /// Current zoning metadata, or `None` while the frontage is unzoned.
    pub zone: Option<ZoneMetadata>,
    /// Residential frontage density per side, normalized to 0.0 through 1.0.
    /// Index 0 is left and index 1 is right.
    pub density: [f32; 2],
}

impl LaneSegment {
    pub fn new(id: u64, road_id: u32, start_id: u32, end_id: u32, length: f32) -> Self {
        Self {
            id,
            road_id,
            start_id,
            end_id,
            length,
            zone: None,
            density: [0.0; 2],
        }
    }

    fn is_residential(&self) -> bool {
        self.zone
            .is_some_and(|zone| zone.zone_type == ZoneType::Residential)
    }
}

/// Lane graph connects lanes with adjacency information to maintain connectivity from intersection topology
#[derive(Debug, Clone, Default, PartialEq)]
pub struct LaneGraph {
    // Mapping from Road ID to its lanes (in both directions)
    pub road_lanes: HashMap<u32, Vec<Lane>>,
    // Adjacency: which lanes touch each intersection
    pub adjacency: HashMap<u32, HashSet<(u32, Direction)>>,
    // Reverse mapping: Lane ID -> Lane info for fast lookup
    pub lane_by_id: HashMap<u64, Lane>,
    // Counter for lane IDs
    pub lane_counter: u64,
    pub road_segments: HashMap<u32, Vec<LaneSegment>>,
    /// Directed lane IDs at each start intersection; unlike road-level adjacency this retains parallel segments.
    pub lane_adjacency: HashMap<u32, HashSet<u64>>,
}

impl LaneGraph {
    /// Create a new empty LaneGraph.
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a road composed of segments, creating lanes for it in both directions.
    /// Returns the allocated Road ID.
    pub fn add_road(&mut self, segments: Vec<(u32, u32, f32)>) -> Option<u32> {
        // Simple sequential assignment for now
        let road_id = self.lane_counter as u32;
        let mut road_lanes = Vec::new();
        let mut road_segments = Vec::new();

        for (start, end, len) in segments {
            road_segments.push(LaneSegment::new(
                self.lane_counter,
                road_id,
                start,
                end,
                len,
            ));
            // Create a forward lane
            let forward_lane = Lane::new(
                self.lane_counter,
                road_id,
                Direction::Forward,
                start,
                end,
                len,
            );
            self.lane_counter += 1;
            self.lane_by_id
                .insert(forward_lane.id, forward_lane.clone());
            self.lane_adjacency
                .entry(start)
                .or_default()
                .insert(forward_lane.id);
            road_lanes.push(forward_lane);

            // Create a backward lane (reverse direction)
            let backward_lane = Lane::new(
                self.lane_counter,
                road_id,
                Direction::Backward,
                end,
                start,
                len,
            );
            self.lane_counter += 1;
            self.lane_by_id
                .insert(backward_lane.id, backward_lane.clone());
            self.lane_adjacency
                .entry(end)
                .or_default()
                .insert(backward_lane.id);
            road_lanes.push(backward_lane);

            // Update adjacency for both endpoints
            self.adjacency
                .entry(start)
                .or_default()
                .insert((road_id, Direction::Forward));
            self.adjacency
                .entry(end)
                .or_default()
                .insert((road_id, Direction::Backward));
        }
        self.road_lanes.insert(road_id, road_lanes);
        self.road_segments.insert(road_id, road_segments);
        Some(road_id)
    }

    /// Remove a road and all its lanes.
    /// Returns true if the road existed.
    pub fn remove_road(&mut self, road_id: u32) -> bool {
        if let Some(lanes) = self.road_lanes.remove(&road_id) {
            self.road_segments.remove(&road_id);
            for lane in &lanes {
                // Remove from adjacency entries
                if let Some(set) = self.adjacency.get_mut(&lane.start_id) {
                    set.remove(&(road_id, lane.direction));
                }
                if let Some(set) = self.adjacency.get_mut(&lane.end_id) {
                    set.remove(&(road_id, lane.direction.opposite()));
                }

                // Remove from lane_by_id mapping
                self.lane_by_id.remove(&lane.id);
                if let Some(ids) = self.lane_adjacency.get_mut(&lane.start_id) {
                    ids.remove(&lane.id);
                    if ids.is_empty() {
                        self.lane_adjacency.remove(&lane.start_id);
                    }
                }
            }
            true
        } else {
            false
        }
    }

    /// Get a reference to a road's lanes.
    pub fn get_road_lanes(&self, road_id: u32) -> Option<&Vec<Lane>> {
        self.road_lanes.get(&road_id)
    }

    /// Get all lanes for a specific intersection (both directions)
    pub fn get_lanes_at_intersection(
        &self,
        intersection_id: u32,
    ) -> Option<&HashSet<(u32, Direction)>> {
        self.adjacency.get(&intersection_id)
    }

    /// Get a lane by its ID
    pub fn get_lane_by_id(&self, id: u64) -> Option<&Lane> {
        self.lane_by_id.get(&id)
    }

    /// Find the least-length directed lane path, including both endpoint lanes.
    /// Invalid or non-positive lane lengths use a cost of 1.0.
    pub fn find_path(&self, start_lane_id: u64, end_lane_id: u64) -> Vec<u64> {
        if !self.lane_by_id.contains_key(&start_lane_id)
            || !self.lane_by_id.contains_key(&end_lane_id)
        {
            return Vec::new();
        }

        if start_lane_id == end_lane_id {
            return vec![start_lane_id];
        }

        // Group lanes by their start intersection once so each lane's outgoing
        // transitions can be enumerated directly from the graph's lane data.
        let mut lanes_from_intersection: HashMap<u32, Vec<u64>> = HashMap::new();
        for lane in self.lane_by_id.values() {
            lanes_from_intersection
                .entry(lane.start_id)
                .or_default()
                .push(lane.id);
        }
        let mut adjacency: HashMap<u64, Vec<u64>> = HashMap::new();
        for lane in self.lane_by_id.values() {
            if let Some(outgoing) = lanes_from_intersection.get(&lane.end_id) {
                adjacency.insert(lane.id, outgoing.clone());
            }
        }

        let mut distances = HashMap::from([(start_lane_id, 0.0_f64)]);
        let mut previous = HashMap::new();
        let mut queue = BinaryHeap::from([PathQueueEntry {
            cost: 0.0,
            lane_id: start_lane_id,
        }]);

        while let Some(PathQueueEntry { cost, lane_id }) = queue.pop() {
            if cost > distances[&lane_id] {
                continue;
            }
            if lane_id == end_lane_id {
                break;
            }

            for &next_id in adjacency.get(&lane_id).into_iter().flatten() {
                let next_lane = &self.lane_by_id[&next_id];
                let lane_cost = if next_lane.length.is_finite() && next_lane.length > 0.0 {
                    f64::from(next_lane.length)
                } else {
                    1.0
                };
                let next_cost = cost + lane_cost;
                if next_cost < *distances.get(&next_id).unwrap_or(&f64::INFINITY) {
                    distances.insert(next_id, next_cost);
                    previous.insert(next_id, lane_id);
                    queue.push(PathQueueEntry {
                        cost: next_cost,
                        lane_id: next_id,
                    });
                }
            }
        }

        if !distances.contains_key(&end_lane_id) {
            return Vec::new();
        }
        let mut path = vec![end_lane_id];
        let mut current = end_lane_id;
        while current != start_lane_id {
            current = previous[&current];
            path.push(current);
        }
        path.reverse();
        path
    }

    /// Add one segment as a road and return its road ID.
    pub fn add_segment(&mut self, start_id: u32, end_id: u32, length: f32) -> Option<u32> {
        self.add_road(vec![(start_id, end_id, length)])
    }

    /// Remove a segment previously added as its own road.
    pub fn remove_segment(&mut self, road_id: u32) -> bool {
        self.remove_road(road_id)
    }

    pub fn get_segments(&self, road_id: u32) -> Option<&[LaneSegment]> {
        self.road_segments.get(&road_id).map(Vec::as_slice)
    }

    /// Assign a land-use designation to a segment on `road_id`, replacing any
    /// previous metadata (including its density cap).
    /// Returns false when the road or segment does not exist.
    pub fn assign_zone(&mut self, road_id: u32, segment_id: u64, zone: ZoneType) -> bool {
        let Some(segment) = self
            .road_segments
            .get_mut(&road_id)
            .and_then(|segments| segments.iter_mut().find(|segment| segment.id == segment_id))
        else {
            return false;
        };
        segment.zone = Some(ZoneMetadata::new(zone));
        true
    }

    /// Get a segment's current designation. `None` means unknown or unzoned.
    pub fn get_zone(&self, road_id: u32, segment_id: u64) -> Option<ZoneType> {
        self.get_zone_metadata(road_id, segment_id)
            .map(|metadata| metadata.zone_type)
    }

    /// Get a segment's full zoning metadata. `None` means unknown or unzoned.
    pub fn get_zone_metadata(&self, road_id: u32, segment_id: u64) -> Option<ZoneMetadata> {
        self.road_segments
            .get(&road_id)?
            .iter()
            .find(|segment| segment.id == segment_id)?
            .zone
    }

    /// Set or clear the density cap (0.0 through 1.0) on a zoned segment.
    /// Returns false for missing/unzoned segments or out-of-range values.
    pub fn set_max_density(
        &mut self,
        road_id: u32,
        segment_id: u64,
        max_density: Option<f32>,
    ) -> bool {
        if max_density.is_some_and(|cap| !cap.is_finite() || !(0.0..=1.0).contains(&cap)) {
            return false;
        }
        let Some(metadata) = self
            .road_segments
            .get_mut(&road_id)
            .and_then(|segments| segments.iter_mut().find(|segment| segment.id == segment_id))
            .and_then(|segment| segment.zone.as_mut())
        else {
            return false;
        };
        metadata.max_density = max_density;
        true
    }

    /// Set residential frontage density to a value from 0.0 through 1.0.
    /// Returns false for non-residential/missing segments or out-of-range values.
    pub fn set_density(&mut self, road_id: u32, segment_id: u64, density: f32) -> bool {
        self.set_side_density(road_id, segment_id, FrontageSide::Left, density)
            && self.set_side_density(road_id, segment_id, FrontageSide::Right, density)
    }

    /// Set density for one residential frontage side.
    pub fn set_side_density(
        &mut self,
        road_id: u32,
        segment_id: u64,
        side: FrontageSide,
        density: f32,
    ) -> bool {
        if !density.is_finite() || !(0.0..=1.0).contains(&density) {
            return false;
        }
        let Some(segment) = self
            .road_segments
            .get_mut(&road_id)
            .and_then(|segments| segments.iter_mut().find(|segment| segment.id == segment_id))
        else {
            return false;
        };
        if !segment.is_residential() {
            return false;
        }
        segment.density[match side {
            FrontageSide::Left => 0,
            FrontageSide::Right => 1,
        }] = density;
        true
    }

    /// Get residential frontage density, or `None` for a missing/non-residential segment.
    pub fn get_density(&self, road_id: u32, segment_id: u64) -> Option<f32> {
        let segment = self
            .road_segments
            .get(&road_id)?
            .iter()
            .find(|segment| segment.id == segment_id)?;
        segment.is_residential().then_some(segment.density[0])
    }

    /// Get density for one residential frontage side.
    pub fn get_side_density(
        &self,
        road_id: u32,
        segment_id: u64,
        side: FrontageSide,
    ) -> Option<f32> {
        let segment = self
            .road_segments
            .get(&road_id)?
            .iter()
            .find(|segment| segment.id == segment_id)?;
        let index = match side {
            FrontageSide::Left => 0,
            FrontageSide::Right => 1,
        };
        segment.is_residential().then_some(segment.density[index])
    }

    pub fn get_lane_ids_at_intersection(&self, intersection_id: u32) -> Option<&HashSet<u64>> {
        self.lane_adjacency.get(&intersection_id)
    }

    /// Get the number of lanes for a specific road
    pub fn num_lanes(&self, road_id: u32) -> usize {
        self.road_lanes.get(&road_id).map_or(0, |lanes| lanes.len())
    }

    /// Verify invariants: every adjacency entry references a road that exists and
    /// contains the intersection with correct direction.
    pub fn check_invariants(&self) -> bool {
        // Check that every lane in road_lanes is consistent with lane_by_id
        for lanes in self.road_lanes.values() {
            for lane in lanes {
                if let Some(lane_in_map) = self.lane_by_id.get(&lane.id) {
                    if *lane_in_map != *lane {
                        return false;
                    }
                } else {
                    return false;
                }
            }
        }

        for lane in self.lane_by_id.values() {
            if !self
                .lane_adjacency
                .get(&lane.start_id)
                .is_some_and(|ids| ids.contains(&lane.id))
            {
                return false;
            }
        }
        for (intersection, ids) in &self.lane_adjacency {
            for id in ids {
                if !self
                    .lane_by_id
                    .get(id)
                    .is_some_and(|lane| lane.start_id == *intersection)
                {
                    return false;
                }
            }
        }

        // Check that adjacency entries reference existing roads
        for (inter, lanes_set) in &self.adjacency {
            // For every lane entry at this intersection, check if it's valid
            for &(road_id, direction) in lanes_set {
                // Check that the road exists
                if !self.road_lanes.contains_key(&road_id) {
                    return false;
                }

                // Verify the road contains a segment that touches this intersection with correct direction
                let lanes = self.road_lanes.get(&road_id).unwrap(); // Safe due to above check
                let mut found = false;
                for lane in lanes {
                    if (direction == Direction::Forward && lane.start_id == *inter)
                        || (direction == Direction::Backward && lane.end_id == *inter)
                    {
                        found = true;
                        break;
                    }
                }
                if !found {
                    return false;
                }
            }
        }

        true
    }

    /// Get number of roads in the graph
    pub fn num_roads(&self) -> usize {
        self.road_lanes.len()
    }

    /// Get number of lanes in the graph  
    pub fn num_lanes_total(&self) -> usize {
        self.lane_by_id.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_and_query_lanes() {
        let mut graph = LaneGraph::new();
        let segs = vec![(1, 2, 100.0), (2, 3, 150.0)];
        let road_id = graph.add_road(segs).expect("add road");
        assert!(graph.get_road_lanes(road_id).is_some());
        assert_eq!(graph.num_lanes(road_id), 4); // 2 segments * 2 directions
        assert!(graph.check_invariants());
    }

    #[test]
    fn test_remove_then_query() {
        let mut graph = LaneGraph::new();
        let segs = vec![(4, 5, 80.0)];
        let road_id = graph.add_road(segs).unwrap();
        assert!(graph.remove_road(road_id));
        assert!(graph.get_road_lanes(road_id).is_none());
        assert_eq!(graph.num_lanes(road_id), 0);
        assert!(graph.check_invariants());
    }

    #[test]
    fn test_duplicate_add() {
        let mut graph = LaneGraph::new();
        let segs1 = vec![(10, 11, 50.0)];
        let segs2 = vec![(12, 13, 60.0)];
        let road_id1 = graph.add_road(segs1).unwrap();
        let road_id2 = graph.add_road(segs2).unwrap();
        assert_ne!(road_id1, road_id2);
        assert_eq!(graph.num_roads(), 2);
        assert_eq!(graph.num_lanes(road_id1), 2);
        assert_eq!(graph.num_lanes(road_id2), 2);
        assert!(graph.check_invariants());
    }

    #[test]
    fn test_invariant_after_multiple_ops() {
        let mut graph = LaneGraph::new();
        let r1 = graph.add_road(vec![(20, 21, 30.0)]).unwrap();
        let r2 = graph.add_road(vec![(21, 22, 40.0)]).unwrap();
        assert!(graph.check_invariants());
        graph.remove_road(r1);
        assert!(graph.check_invariants());
        graph.remove_road(r2);
        assert!(graph.check_invariants());
    }

    #[test]
    fn test_lane_lookup() {
        let mut graph = LaneGraph::new();
        let segs = vec![(20, 21, 30.0)];
        let road_id = graph.add_road(segs).unwrap();

        // Test that we can look up lanes by ID
        let roads_lanes = graph.get_road_lanes(road_id).unwrap();
        for lane in roads_lanes {
            assert!(graph.get_lane_by_id(lane.id).is_some());
        }
    }

    #[test]
    fn test_direction_correctness() {
        let mut graph = LaneGraph::new();
        let segs = vec![(100, 101, 50.0)];
        let road_id = graph.add_road(segs).unwrap();

        let roads_lanes = graph.get_road_lanes(road_id).unwrap();
        assert_eq!(roads_lanes.len(), 2); // One lane in each direction

        // First lane (forward)
        let forward_lane = &roads_lanes[0];
        assert_eq!(forward_lane.direction, Direction::Forward);
        assert_eq!(forward_lane.start_id, 100);
        assert_eq!(forward_lane.end_id, 101);

        // Second lane (backward)
        let backward_lane = &roads_lanes[1];
        assert_eq!(backward_lane.direction, Direction::Backward);
        assert_eq!(backward_lane.start_id, 101);
        assert_eq!(backward_lane.end_id, 100);
    }

    #[test]
    fn test_multiple_segments() {
        let mut graph = LaneGraph::new();
        let segs = vec![(10, 20, 50.0), (20, 30, 60.0)];
        let road_id = graph.add_road(segs).unwrap();
        assert_eq!(graph.num_lanes(road_id), 4); // 2 segments * 2 directions each

        // Check lane connectivity via adjacency
        let lanes_at_start = graph.get_lanes_at_intersection(10).unwrap();
        assert!(lanes_at_start.contains(&(road_id, Direction::Forward)));

        let lanes_at_end = graph.get_lanes_at_intersection(30).unwrap();
        assert!(lanes_at_end.contains(&(road_id, Direction::Backward)));
    }

    #[test]
    fn segment_adjacency_preserves_parallel_segments_and_cleans_up() {
        let mut graph = LaneGraph::new();
        let first = graph.add_segment(1, 2, 10.0).unwrap();
        let second = graph.add_segment(1, 3, 12.0).unwrap();
        assert_eq!(graph.get_segments(first).unwrap().len(), 1);
        assert_eq!(graph.get_lane_ids_at_intersection(1).unwrap().len(), 2);
        assert!(graph.check_invariants());

        assert!(graph.remove_segment(first));
        let remaining = graph.get_lane_ids_at_intersection(1).unwrap();
        assert_eq!(remaining.len(), 1);
        assert!(graph.get_road_lanes(second).is_some());
        assert!(graph.check_invariants());
        assert!(graph.remove_segment(second));
        assert!(graph.get_lane_ids_at_intersection(1).is_none());
        assert!(graph.check_invariants());
    }

    #[test]
    fn segment_zone_can_be_assigned_changed_and_retrieved() {
        let mut graph = LaneGraph::new();
        let road_id = graph.add_road(vec![(1, 2, 10.0), (2, 3, 12.0)]).unwrap();
        let segments = graph.get_segments(road_id).unwrap();
        let first_segment = segments[0].id;
        let second_segment = segments[1].id;

        assert_eq!(graph.get_zone(road_id, first_segment), None);
        assert!(graph.assign_zone(road_id, first_segment, ZoneType::Residential));
        assert_eq!(
            graph.get_zone(road_id, first_segment),
            Some(ZoneType::Residential)
        );
        assert!(graph.assign_zone(road_id, first_segment, ZoneType::Commercial));
        assert_eq!(
            graph.get_zone(road_id, first_segment),
            Some(ZoneType::Commercial)
        );
        assert_eq!(graph.get_zone(road_id, second_segment), None);
        assert!(!graph.assign_zone(road_id, u64::MAX, ZoneType::Industrial));
        assert!(!graph.assign_zone(u32::MAX, first_segment, ZoneType::Industrial));
    }

    #[test]
    fn zone_metadata_is_stored_per_segment() {
        let mut graph = LaneGraph::new();
        let road_id = graph.add_road(vec![(1, 2, 10.0), (2, 3, 12.0)]).unwrap();
        let segments = graph.get_segments(road_id).unwrap();
        let office = segments[0].id;
        let unzoned = segments[1].id;

        assert!(!graph.set_max_density(road_id, unzoned, Some(0.5)));
        assert!(graph.assign_zone(road_id, office, ZoneType::Office));
        assert_eq!(
            graph.get_zone_metadata(road_id, office),
            Some(ZoneMetadata::new(ZoneType::Office))
        );
        assert!(graph.set_max_density(road_id, office, Some(0.4)));
        assert_eq!(
            graph.get_segments(road_id).unwrap()[0].zone,
            Some(ZoneMetadata {
                zone_type: ZoneType::Office,
                max_density: Some(0.4),
            })
        );
        assert_eq!(graph.get_zone_metadata(road_id, unzoned), None);
        assert!(!graph.set_max_density(road_id, office, Some(1.5)));
        assert!(!graph.set_max_density(road_id, office, Some(f32::NAN)));
        assert!(graph.set_max_density(road_id, office, None));
        assert_eq!(
            graph
                .get_zone_metadata(road_id, office)
                .unwrap()
                .max_density,
            None
        );

        // Rezoning replaces the metadata, including any cap.
        assert!(graph.set_max_density(road_id, office, Some(0.8)));
        assert!(graph.assign_zone(road_id, office, ZoneType::Farm));
        assert_eq!(
            graph.get_zone_metadata(road_id, office),
            Some(ZoneMetadata::new(ZoneType::Farm))
        );
    }

    #[test]
    fn residential_segment_density_is_normalized_and_zone_scoped() {
        let mut graph = LaneGraph::new();
        let road_id = graph.add_road(vec![(1, 2, 10.0), (2, 3, 12.0)]).unwrap();
        let segments = graph.get_segments(road_id).unwrap();
        let residential = segments[0].id;
        let unzoned = segments[1].id;

        assert_eq!(graph.get_density(road_id, residential), None);
        assert!(graph.assign_zone(road_id, residential, ZoneType::Residential));
        assert_eq!(graph.get_density(road_id, residential), Some(0.0));
        assert!(graph.set_side_density(road_id, residential, FrontageSide::Left, 0.65));
        assert!(graph.set_side_density(road_id, residential, FrontageSide::Right, 0.35));
        assert_eq!(
            graph.get_side_density(road_id, residential, FrontageSide::Left),
            Some(0.65)
        );
        assert_eq!(
            graph.get_side_density(road_id, residential, FrontageSide::Right),
            Some(0.35)
        );
        assert!(graph.set_density(road_id, residential, 1.0));
        assert_eq!(
            graph.get_side_density(road_id, residential, FrontageSide::Left),
            Some(1.0)
        );
        assert_eq!(
            graph.get_side_density(road_id, residential, FrontageSide::Right),
            Some(1.0)
        );
        assert!(!graph.set_density(road_id, residential, -0.1));
        assert!(!graph.set_density(road_id, residential, 1.1));
        assert!(!graph.set_density(road_id, residential, f32::NAN));
        assert!(!graph.set_density(road_id, unzoned, 0.5));
        assert_eq!(graph.get_density(road_id, unzoned), None);
    }

    #[test]
    fn assigning_residential_stores_zone_type_and_density() {
        let mut graph = LaneGraph::new();
        let road_id = graph.add_road(vec![(1, 2, 10.0)]).unwrap();
        let segment_id = graph.get_segments(road_id).unwrap()[0].id;

        assert!(graph.assign_zone(road_id, segment_id, ZoneType::Residential));
        assert!(graph.set_max_density(road_id, segment_id, Some(0.6)));
        assert!(graph.set_density(road_id, segment_id, 0.4));

        let segment = &graph.get_segments(road_id).unwrap()[0];
        assert_eq!(
            segment.zone,
            Some(ZoneMetadata {
                zone_type: ZoneType::Residential,
                max_density: Some(0.6),
            })
        );
        assert_eq!(segment.density, [0.4, 0.4]);
        assert_eq!(
            graph.get_zone(road_id, segment_id),
            Some(ZoneType::Residential)
        );
        assert_eq!(graph.get_density(road_id, segment_id), Some(0.4));
    }

    #[test]
    fn find_path_returns_shortest_connected_lane_sequence() {
        let mut graph = LaneGraph::new();
        graph.add_segment(0, 1, 1.0).unwrap();
        graph.add_segment(1, 2, 1.0).unwrap();
        graph.add_segment(1, 3, 5.0).unwrap();
        graph.add_segment(2, 3, 1.0).unwrap();
        graph.add_segment(3, 4, 1.0).unwrap();
        let start = graph
            .lane_by_id
            .values()
            .find(|lane| lane.start_id == 0 && lane.end_id == 1)
            .unwrap()
            .id;
        let end = graph
            .lane_by_id
            .values()
            .find(|lane| lane.start_id == 3 && lane.end_id == 4)
            .unwrap()
            .id;
        let first_short = graph
            .lane_by_id
            .values()
            .find(|lane| lane.start_id == 1 && lane.end_id == 2)
            .unwrap()
            .id;
        let via_short = graph
            .lane_by_id
            .values()
            .find(|lane| lane.start_id == 2 && lane.end_id == 3)
            .unwrap()
            .id;

        assert_eq!(
            graph.find_path(start, end),
            vec![start, first_short, via_short, end]
        );
    }

    #[test]
    fn find_path_handles_single_lane_unknown_and_disconnected_endpoints() {
        let mut graph = LaneGraph::new();
        graph.add_segment(1, 2, 5.0).unwrap();
        graph.add_segment(8, 9, 5.0).unwrap();
        let first = graph
            .lane_by_id
            .values()
            .find(|lane| lane.start_id == 1 && lane.end_id == 2)
            .unwrap()
            .id;
        let disconnected = graph
            .lane_by_id
            .values()
            .find(|lane| lane.start_id == 8 && lane.end_id == 9)
            .unwrap()
            .id;

        assert_eq!(graph.find_path(first, first), vec![first]);
        assert!(graph.find_path(first, disconnected).is_empty());
        assert!(graph.find_path(u64::MAX, first).is_empty());
    }

    #[test]
    fn find_path_handles_dense_intersection_connectivity() {
        let mut graph = LaneGraph::new();
        for end in 2..12 {
            graph.add_segment(1, end, 1.0).unwrap();
        }
        let start = graph
            .lane_by_id
            .values()
            .find(|lane| lane.start_id == 1 && lane.end_id == 2)
            .unwrap()
            .id;
        let end = graph
            .lane_by_id
            .values()
            .find(|lane| lane.start_id == 2 && lane.end_id == 1)
            .unwrap()
            .id;

        let path = graph.find_path(start, end);
        assert_eq!(path.first(), Some(&start));
        assert_eq!(path.last(), Some(&end));
        assert!(path.windows(2).all(|pair| {
            graph.lane_by_id[&pair[0]].end_id == graph.lane_by_id[&pair[1]].start_id
        }));
    }
}
