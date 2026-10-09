use tj_network::LaneGraph;
use tj_traffic::TrafficSimulation;

#[test]
fn advances_two_hundred_vehicles_for_one_hundred_ticks() {
    let mut network = LaneGraph::new();
    let road_id = network
        .add_segment(1, 2, 100.0)
        .expect("road should be added");
    let lane_id = network.get_road_lanes(road_id).unwrap()[0].id;
    let mut simulation = TrafficSimulation::new(network);

    for _ in 0..200 {
        simulation
            .spawn_vehicle(lane_id, 1.0)
            .expect("vehicle should spawn on the lane");
    }

    for _ in 0..100 {
        simulation.tick();
    }

    assert_eq!(simulation.tick_count(), 100);
    assert_eq!(simulation.vehicles().len(), 200);
    assert!(simulation.vehicles().iter().all(|vehicle| {
        vehicle.lane_id == lane_id
            && vehicle.position.is_finite()
            && (0.0..=100.0).contains(&vehicle.position)
    }));
}
