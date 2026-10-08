// Graph integrity tests: after every add/remove operation the lane graph must
// hold no duplicate edges and every live intersection must stay reachable.

use std::collections::{HashMap, HashSet, VecDeque};
use tj_network::LaneGraph;

/// Assert no lane (directed edge) appears twice: lane IDs are unique across
/// roads, every lane is indexed exactly once, and no road repeats a
/// (start, end, direction) edge.
fn assert_no_duplicate_edges(graph: &LaneGraph) {
    let mut ids = HashSet::new();
    let mut lane_count = 0;
    for (road_id, lanes) in &graph.road_lanes {
        let mut edges = HashSet::new();
        for lane in lanes {
            lane_count += 1;
            assert!(ids.insert(lane.id), "duplicate lane id {}", lane.id);
            assert!(
                edges.insert((lane.start_id, lane.end_id, lane.direction)),
                "road {road_id} repeats edge {}->{}",
                lane.start_id,
                lane.end_id
            );
        }
    }
    assert_eq!(graph.lane_by_id.len(), lane_count);
    let indexed: usize = graph.lane_adjacency.values().map(HashSet::len).sum();
    assert_eq!(indexed, lane_count, "stale lane adjacency");
    assert!(graph.check_invariants());
}

/// True when every intersection touched by a live lane is reachable from any other.
fn is_connected(graph: &LaneGraph) -> bool {
    let mut neighbours: HashMap<u32, Vec<u32>> = HashMap::new();
    for lane in graph.lane_by_id.values() {
        neighbours
            .entry(lane.start_id)
            .or_default()
            .push(lane.end_id);
        neighbours
            .entry(lane.end_id)
            .or_default()
            .push(lane.start_id);
    }
    let Some(&start) = neighbours.keys().next() else {
        return true;
    };
    let mut seen = HashSet::from([start]);
    let mut queue = VecDeque::from([start]);
    while let Some(node) = queue.pop_front() {
        for &next in &neighbours[&node] {
            if seen.insert(next) {
                queue.push_back(next);
            }
        }
    }
    seen.len() == neighbours.len()
}

fn assert_integrity(graph: &LaneGraph) {
    assert_no_duplicate_edges(graph);
    assert!(is_connected(graph), "graph became disconnected");
}

#[test]
fn graph_stays_duplicate_free_and_connected_after_each_add_and_remove() {
    let mut graph = LaneGraph::new();
    assert_integrity(&graph);

    // Grow a connected network: a 1-2-3-4 chain, a branch at 2, a loop 4-1,
    // and a multi-segment road 3-6-7. Every add attaches to an existing node.
    let segments = [
        (1, 2, 10.0),
        (2, 3, 12.0),
        (3, 4, 8.0),
        (2, 5, 5.0),
        (4, 1, 9.0),
    ];
    let mut roads = Vec::new();
    for (start, end, len) in segments {
        roads.push(graph.add_segment(start, end, len).unwrap());
        assert_integrity(&graph);
    }
    roads.push(graph.add_road(vec![(3, 6, 4.0), (6, 7, 4.0)]).unwrap());
    assert_integrity(&graph);

    // Re-adding an existing edge creates a distinct parallel road, never a
    // duplicate lane.
    let parallel = graph.add_segment(1, 2, 10.0).unwrap();
    assert_integrity(&graph);
    assert_eq!(graph.num_lanes_total(), 16);

    // Remove in an order that keeps the network connected: the parallel road,
    // the loop edge (4-1 still reachable via 3), then leaves back to the core.
    assert!(graph.remove_road(parallel));
    assert_integrity(&graph);
    for i in [4, 5, 3, 2, 1, 0] {
        assert!(graph.remove_road(roads[i]));
        assert_integrity(&graph);
    }
    assert_eq!(graph.num_lanes_total(), 0);
    assert!(graph.lane_adjacency.is_empty());
}

#[test]
fn removing_a_bridge_is_detected_as_disconnected() {
    // Guards the connectivity check itself: it must flag a split network.
    let mut graph = LaneGraph::new();
    graph.add_segment(1, 2, 1.0).unwrap();
    let bridge = graph.add_segment(2, 3, 1.0).unwrap();
    graph.add_segment(3, 4, 1.0).unwrap();
    assert_integrity(&graph);

    assert!(graph.remove_road(bridge));
    assert_no_duplicate_edges(&graph);
    assert!(!is_connected(&graph));
}

#[test]
fn removing_a_missing_road_leaves_graph_unchanged() {
    let mut graph = LaneGraph::new();
    let road = graph.add_segment(1, 2, 1.0).unwrap();
    let before = graph.clone();
    assert!(!graph.remove_road(road + 1000));
    assert_eq!(graph, before);
    assert_integrity(&graph);
}
