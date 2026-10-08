use proptest::prelude::*;
use std::collections::HashMap;
use tj_network::{LaneGraph, ZoneType};

fn zone_component_count(graph: &LaneGraph, zones: &[u32]) -> usize {
    let mut parent: HashMap<u32, u32> = HashMap::new();
    for lanes in graph.road_lanes.values() {
        for lane in lanes {
            for node in [lane.start_id, lane.end_id] {
                if let std::collections::hash_map::Entry::Vacant(entry) = parent.entry(node) {
                    entry.insert(node);
                }
            }
        }
    }

    fn root(parent: &mut HashMap<u32, u32>, mut node: u32) -> u32 {
        while parent[&node] != node {
            let grandparent = parent[&parent[&node]];
            parent.insert(node, grandparent);
            node = grandparent;
        }
        node
    }

    for lanes in graph.road_lanes.values() {
        for lane in lanes
            .iter()
            .filter(|lane| lane.direction == tj_network::Direction::Forward)
        {
            let left = root(&mut parent, lane.start_id);
            let right = root(&mut parent, lane.end_id);
            parent.insert(left, right);
        }
    }

    zones
        .iter()
        .map(|&node| root(&mut parent, node))
        .collect::<std::collections::HashSet<_>>()
        .len()
}

proptest! {
    #[test]
    fn removing_a_road_from_a_zoned_cycle_never_increases_zone_components(
        extra_nodes in 0u8..32,
        removal_index in 0u8..32,
    ) {
        let node_count = usize::from(extra_nodes) + 3;
        let mut graph = LaneGraph::new();
        let mut road_ids = Vec::with_capacity(node_count);
        for node in 0..node_count {
            let next = (node + 1) % node_count;
            let road = graph.add_road(vec![(node as u32, next as u32, 1.0)]).unwrap();
            road_ids.push(road);
            if node < 3 {
                let segment = graph.get_segments(road).unwrap()[0].id;
                prop_assert!(graph.assign_zone(road, segment, ZoneType::Residential));
            }
        }

        let zones = [0, 1, 2];
        let before = zone_component_count(&graph, &zones);
        let road_to_remove = road_ids[usize::from(removal_index) % node_count];
        prop_assert!(graph.remove_road(road_to_remove));
        let after = zone_component_count(&graph, &zones);
        prop_assert!(after <= before, "zone components increased from {before} to {after}");
    }
}
