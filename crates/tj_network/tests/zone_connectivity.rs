use std::collections::{HashMap, HashSet, VecDeque};
use tj_network::{LaneGraph, ZoneType};

/// Counts undirected connected components that contain at least one node
/// incident to a currently zone-assigned segment.
fn zone_component_count(graph: &LaneGraph) -> usize {
    let mut neighbors: HashMap<u32, HashSet<u32>> = HashMap::new();
    // Include every live lane, so both lane directions and parallel roads are
    // represented correctly as undirected physical connectivity.
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

/// Exhaustively checks every zoning layout and every single-road removal for
/// ring networks of 3 through 8 roads. For each graph, the removal loop is
/// universal over all live roads rather than selecting a random candidate.
#[test]
fn removing_any_road_never_increases_zone_components() {
    for road_count in 3u32..=8 {
        let zoning_layout_count = 1u32 << road_count;
        for zoning_mask in 0..zoning_layout_count {
            let mut graph = LaneGraph::new();
            let mut road_ids = Vec::with_capacity(road_count as usize);

            for road_index in 0..road_count {
                let next_node = (road_index + 1) % road_count;
                let road_id = graph
                    .add_road(vec![(road_index, next_node, 1.0)])
                    .expect("ring road should be added");
                road_ids.push(road_id);
                if zoning_mask & (1 << road_index) != 0 {
                    let segment_id = graph.get_segments(road_id).unwrap()[0].id;
                    assert!(graph.assign_zone(road_id, segment_id, ZoneType::Residential));
                }
            }

            let before = zone_component_count(&graph);
            for road_id in road_ids {
                let mut after_graph = graph.clone();
                assert!(after_graph.remove_road(road_id));
                let after = zone_component_count(&after_graph);
                assert!(
                    after <= before,
                    "ring with {road_count} roads and zoning mask {zoning_mask:#b}: removing road {road_id} increased zone components from {before} to {after}"
                );
            }
        }
    }
}
