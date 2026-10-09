//! Headless integration coverage for staging a road before approving it.

use tj_network::planning::PlanningOverlay;
use tj_network::LaneGraph;

#[test]
fn staged_road_does_not_change_live_network_until_approved() {
    let live_network = LaneGraph::new();
    let mut planning = PlanningOverlay::new(live_network);
    planning.enable();

    let staged_road = planning
        .propose_add_road(vec![(10, 11, 25.0)])
        .expect("planning mode should stage a valid road");

    assert_eq!(planning.get_live_graph().num_roads(), 0);
    assert!(!planning
        .get_live_graph()
        .road_lanes
        .contains_key(&staged_road));
    assert_eq!(planning.get_overlay_graph().num_roads(), 1);
    assert!(planning
        .get_overlay_graph()
        .road_lanes
        .contains_key(&staged_road));
    assert_eq!(planning.pending_change_count(), 1);

    planning
        .approve_change(0)
        .expect("the staged road should be approved");

    assert_eq!(planning.get_live_graph().num_roads(), 1);
    assert!(planning
        .get_live_graph()
        .road_lanes
        .contains_key(&staged_road));
    assert_eq!(planning.pending_change_count(), 0);
}
