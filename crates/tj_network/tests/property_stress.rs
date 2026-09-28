// Property and stress tests for LaneGraph.
//
// Origin: these were written on card t_99925683 (macdev) against an earlier
// `NetworkGraph` API that never landed on master. The intent is preserved, but
// every call targets the real `LaneGraph` surface, and the invariant checked is
// the one master actually maintains: `check_invariants()` after every mutation.
//
// The original also wrote a timing line into docs/SCALING.md from inside the
// test. That is a side effect in a test, and it makes the test order-dependent
// on the filesystem. Dropped: this file reports timing to stdout instead.

use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};
use tj_network::{Direction, LaneGraph};

/// A road is a list of segments as `(start_node, end_node, length)`.
fn random_road(rng: &mut StdRng) -> Vec<(u32, u32, f32)> {
    let seg_count = rng.gen_range(1..=4);
    let mut node = rng.gen_range(1..50);
    let mut segments = Vec::with_capacity(seg_count);
    for _ in 0..seg_count {
        let next = node + rng.gen_range(1..=5);
        let length: f32 = rng.gen_range(1.0..100.0);
        segments.push((node, next, length));
        node = next;
    }
    segments
}

/// Drive a random add/remove sequence and assert the graph's own invariant
/// checker agrees after every single operation. This is the real test: it does
/// not trust the mutation to be correct, it asks the graph.
fn run_random_operations(seed: u64, ops: usize) -> bool {
    let mut rng = StdRng::seed_from_u64(seed);
    let mut graph = LaneGraph::new();
    let mut live: Vec<u32> = Vec::new();

    for _ in 0..ops {
        // Bias toward adds so the graph actually grows instead of thrashing.
        if rng.gen_bool(0.7) || live.is_empty() {
            // add_road returns the allocated Road ID, not a count.
            if let Some(road_id) = graph.add_road(random_road(&mut rng)) {
                live.push(road_id);
            }
        } else {
            let idx = rng.gen_range(0..live.len());
            let road_id = live[idx];
            // Removing must report success for a road we know exists.
            if !graph.remove_road(road_id) {
                return false;
            }
            live.swap_remove(idx);
        }

        if !graph.check_invariants() {
            return false;
        }
    }
    true
}

#[test]
fn property_invariants_hold_under_random_ops() {
    assert!(run_random_operations(0xdead_beef, 2_000));
}

#[test]
fn property_invariants_hold_under_a_different_seed() {
    // A second seed so a single lucky seed cannot carry the suite.
    assert!(run_random_operations(0x5eed_1234, 2_000));
}

#[test]
fn property_removing_an_absent_road_is_false_and_harmless() {
    let mut graph = LaneGraph::new();
    let before = graph.num_roads();
    assert!(!graph.remove_road(9_999));
    assert_eq!(graph.num_roads(), before);
    assert!(graph.check_invariants());
}

#[test]
fn property_road_count_tracks_operations() {
    let mut graph = LaneGraph::new();
    let mut live = Vec::new();
    for i in 0..500 {
        let id = graph
            .add_road(vec![(i, i + 1, 10.0), (i + 1, i + 2, 10.0)])
            .expect("valid geometry must allocate a road id");
        live.push(id);
    }
    assert_eq!(graph.num_roads(), 500);
    // Two segments per road means two lanes per segment: 4 lanes per road.
    assert_eq!(graph.num_lanes_total(), 500 * 4);

    for (i, id) in live.iter().enumerate() {
        assert!(graph.remove_road(*id));
        if i % 7 == 0 {
            assert!(graph.check_invariants());
        }
    }
    assert_eq!(graph.num_roads(), 0);
    assert_eq!(graph.num_lanes_total(), 0);
    assert!(graph.check_invariants());
}

#[test]
fn property_lane_lookup_stays_consistent_after_removal() {
    let mut graph = LaneGraph::new();
    let id = graph.add_road(vec![(1, 2, 25.0)]).unwrap();
    let lanes = graph.get_road_lanes(id).unwrap().clone();
    let lane_ids: Vec<u64> = lanes.iter().map(|l| l.id).collect();
    for lane_id in &lane_ids {
        assert!(graph.get_lane_by_id(*lane_id).is_some());
    }

    assert!(graph.remove_road(id));
    for lane_id in &lane_ids {
        assert!(
            graph.get_lane_by_id(*lane_id).is_none(),
            "lane {lane_id} outlived its road"
        );
    }
}

#[test]
fn property_opposite_is_an_involution() {
    assert_eq!(Direction::Forward.opposite(), Direction::Backward);
    assert_eq!(Direction::Backward.opposite(), Direction::Forward);
}

#[test]
fn stress_100k_roads_scale() {
    let mut graph = LaneGraph::new();
    let start = std::time::Instant::now();
    for i in 0..100_000u32 {
        graph.add_road(vec![(i, i + 1, 10.0)]);
    }
    let elapsed = start.elapsed();
    // Reported to stdout, not written into the docs tree from a test.
    println!("added 100k roads in {elapsed:?}");
    assert_eq!(graph.num_roads(), 100_000);
    assert!(graph.check_invariants());
}
