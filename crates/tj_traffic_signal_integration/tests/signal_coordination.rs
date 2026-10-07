use tj_network::LaneGraph;
use tj_traffic::{SignalState, TrafficSignal, TrafficSimulation};

#[test]
fn traffic_signal_controls_vehicle_flow_through_the_simulation_loop() {
    // A vehicle travels from intersection 1 to the signalized intersection 2.
    let mut network = LaneGraph::new();
    let road_id = network
        .add_road(vec![(1, 2, 10.0)])
        .expect("road should be created");
    let lane_id = network
        .get_road_lanes(road_id)
        .expect("road lanes should exist")
        .iter()
        .find(|lane| lane.start_id == 1 && lane.end_id == 2)
        .expect("forward lane should exist")
        .id;

    let mut simulation = TrafficSimulation::new(network);
    simulation.add_signal(2, TrafficSignal::new(3, 2));
    let vehicle_id = simulation
        .spawn_vehicle(lane_id, 2.0)
        .expect("vehicle should spawn on the road");

    // The vehicle travels along its lane while red, but cannot cross the stop line.
    for expected_position in [2.0, 4.0, 6.0] {
        simulation.tick();
        assert_eq!(
            simulation
                .vehicle(vehicle_id)
                .expect("vehicle exists")
                .position,
            expected_position
        );
    }

    // Tick 3 is green: the vehicle advances toward the end of its lane.
    assert_eq!(TrafficSignal::new(3, 2).state_at(3), SignalState::Green);
    simulation.tick();
    assert_eq!(
        simulation
            .vehicle(vehicle_id)
            .expect("vehicle exists")
            .position,
        8.0
    );

    // The next green tick reaches the end of the lane.
    simulation.tick();
    assert_eq!(
        simulation
            .vehicle(vehicle_id)
            .expect("vehicle exists")
            .position,
        10.0
    );
}
