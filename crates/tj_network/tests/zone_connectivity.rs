use proptest::prelude::*;
use std::collections::{HashMap, HashSet, VecDeque};
use tj_network::{LaneGraph, ZoneType};

/// Counts undirected connected components that contain at least one endpoint
/// of a live zone-assigned segment.
fn zone_component_count(graph: &LaneGraph) -> usize {
    let mut neighbors: HashMap<u32, HashSet<u32>> = HashMap::new();
    for lane in graph.lane_by_id.values() {
        neighbors
            .entry(lane.start_id)
            .or_default()
            .insert(lane.end_id);
        neighbors
            .entry(lane.end_id)
            .or_default()
            .insert(lane.start_id);
    }

    let zone_nodes: HashSet<u32> = graph
        .road_segments
        .values()
        .flatten()
        .filter(|segment| segment.zone.is_some())
        .flat_map(|segment| [segment.start_id, segment.end_id])
        .collect();

    let mut visited = HashSet::new();
    let mut count = 0;
    for &node in neighbors.keys() {
        if !visited.insert(node) {
            continue;
        }
        let mut pending = VecDeque::from([node]);
        let mut contains_zone = zone_nodes.contains(&node);
        while let Some(current) = pending.pop_front() {
            for &next in neighbors.get(&current).into_iter().flatten() {
                if visited.insert(next) {
                    contains_zone |= zone_nodes.contains(&next);
                    pending.push_back(next);
                }
            }
        }
        count += usize::from(contains_zone);
    }
    count
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(256))]

    /// For each generated network and zoning layout, check every road removal.
    /// A ring backbone ensures each physical road belongs to a cycle, so no
    /// single removal can split the underlying network into two zone-bearing
    /// components. Extra roads vary the topology, including parallel roads.
    #[test]
    fn removing_any_road_never_increases_zone_components(
        node_count in 3u32..9,
        extra_roads in prop::collection::vec((0u32..8, 0u32..8, any::<bool>()), 0..17),
        ring_zones in prop::collection::vec(any::<bool>(), 3..9),
    ) {
        let mut graph = LaneGraph::new();
        let mut road_ids = Vec::new();
        let mut roads = Vec::new();
        for node in 0..node_count {
            roads.push((node, (node + 1) % node_count, ring_zones[node as usize % ring_zones.len()]));
        }
        roads.extend(extra_roads.into_iter().filter(|(start, end, _)| {
            *start < node_count && *end < node_count && start != end
        }));

        for (start, end, zoned) in roads {
            let road_id = graph.add_segment(start, end, 1.0).expect("road should be added");
            if zoned {
                let segment_id = graph.get_segments(road_id).unwrap()[0].id;
                prop_assert!(graph.assign_zone(road_id, segment_id, ZoneType::Residential));
            }
            road_ids.push(road_id);
        }

        let before = zone_component_count(&graph);
        for road_id in road_ids {
            let mut after_graph = graph.clone();
            prop_assert!(after_graph.remove_road(road_id));
            let after = zone_component_count(&after_graph);
            prop_assert!(
                after <= before,
                "removing road {road_id} increased zone components from {before} to {after}"
            );
        }
    }
}
