use proptest::prelude::*;
use std::collections::{HashMap, HashSet, VecDeque};
use tj_network::{LaneGraph, ZoneType};

/// Counts undirected components containing an endpoint of a live zoned segment.
fn zone_component_count(graph: &LaneGraph) -> usize {
    let mut neighbors: HashMap<u32, HashSet<u32>> = HashMap::new();
    let mut zone_nodes = HashSet::new();

    for segments in graph.road_segments.values() {
        for segment in segments {
            // Include both endpoints, even when they have no other incident road.
            neighbors.entry(segment.start_id).or_default();
            neighbors.entry(segment.end_id).or_default();
            neighbors
                .entry(segment.start_id)
                .or_default()
                .insert(segment.end_id);
            neighbors
                .entry(segment.end_id)
                .or_default()
                .insert(segment.start_id);
            if segment.zone.is_some() {
                zone_nodes.insert(segment.start_id);
                zone_nodes.insert(segment.end_id);
            }
        }
    }

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

    /// For every generated graph, quantify over every physical road removal.
    #[test]
    fn removing_any_road_never_increases_zone_components(
        extra_roads in prop::collection::vec((0u32..8, 0u32..8, any::<bool>()), 0..25),
        ring_zones in prop::collection::vec(any::<bool>(), 3..9),
    ) {
        let mut graph = LaneGraph::new();
        let mut road_ids = Vec::new();
        let node_count = ring_zones.len() as u32;
        let roads = (0..node_count)
            .map(|node| (node, (node + 1) % node_count, ring_zones[node as usize]))
            .chain(extra_roads.into_iter().filter(|(start, end, _)| {
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
