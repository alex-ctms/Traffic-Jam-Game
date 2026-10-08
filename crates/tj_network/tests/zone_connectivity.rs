use proptest::prelude::*;
use std::collections::{HashMap, HashSet, VecDeque};
use tj_network::{LaneGraph, ZoneType};

/// Counts undirected connected components that contain at least one zoned node.
/// A node is zoned when it is an endpoint of a zone-assigned road segment.
fn zone_component_count(graph: &LaneGraph) -> usize {
    let mut neighbors: HashMap<u32, HashSet<u32>> = HashMap::new();
    for lanes in graph.road_lanes.values() {
        for lane in lanes {
            neighbors
                .entry(lane.start_id)
                .or_default()
                .insert(lane.end_id);
            neighbors
                .entry(lane.end_id)
                .or_default()
                .insert(lane.start_id);
        }
    }

    let zone_nodes: HashSet<u32> = graph
        .road_segments
        .values()
        .flatten()
        .filter(|segment| segment.zone.is_some())
        .flat_map(|segment| [segment.start_id, segment.end_id])
        .collect();

    let mut visited = HashSet::new();
    let mut components_with_zones = 0;
    for &node in neighbors.keys() {
        if !visited.insert(node) {
            continue;
        }

        let mut contains_zone = zone_nodes.contains(&node);
        let mut pending = VecDeque::from([node]);
        while let Some(current) = pending.pop_front() {
            if let Some(adjacent) = neighbors.get(&current) {
                for &next in adjacent {
                    if visited.insert(next) {
                        contains_zone |= zone_nodes.contains(&next);
                        pending.push_back(next);
                    }
                }
            }
        }
        components_with_zones += usize::from(contains_zone);
    }
    components_with_zones
}

proptest! {
    #[test]
    fn removing_any_road_never_increases_zone_components(
        node_count in 3u8..32,
        zoned_roads in prop::collection::vec(any::<bool>(), 3..32),
    ) {
        let mut graph = LaneGraph::new();
        let mut road_ids = Vec::with_capacity(usize::from(node_count));
        for node in 0..node_count {
            let next = (node + 1) % node_count;
            let road = graph.add_road(vec![(u32::from(node), u32::from(next), 1.0)]).unwrap();
            road_ids.push(road);
            if zoned_roads[usize::from(node) % zoned_roads.len()] {
                let segment_id = graph.get_segments(road).unwrap()[0].id;
                prop_assert!(graph.assign_zone(road, segment_id, ZoneType::Residential));
            }
        }

        let before = zone_component_count(&graph);
        for road_id in road_ids {
            let mut graph_after_removal = graph.clone();
            prop_assert!(graph_after_removal.remove_road(road_id));
            let after = zone_component_count(&graph_after_removal);
            prop_assert!(
                after <= before,
                "removing road {road_id} increased zone-containing components from {before} to {after}"
            );
        }
    }
}
