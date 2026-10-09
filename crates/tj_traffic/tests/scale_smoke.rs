use std::panic::{catch_unwind, AssertUnwindSafe};
use std::sync::mpsc;
use std::time::Duration;
use tj_network::LaneGraph;
use tj_traffic::TrafficSimulation;

#[test]
fn advances_two_hundred_vehicles_for_one_hundred_ticks() {
    // Run on a worker so a deadlocked tick cannot hang the entire test suite.
    // A panic is caught and reported through the channel for an explicit failure.
    let (result_tx, result_rx) = mpsc::channel();
    std::thread::spawn(move || {
        let result = catch_unwind(AssertUnwindSafe(|| {
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
        }));
        let _ = result_tx.send(result);
    });

    match result_rx.recv_timeout(Duration::from_secs(30)) {
        Ok(Ok(())) => {}
        Ok(Err(_)) => panic!("200-vehicle simulation panicked during its 100 ticks"),
        Err(mpsc::RecvTimeoutError::Timeout) => {
            panic!("200-vehicle simulation did not finish 100 ticks within 30 seconds (possible deadlock)");
        }
        Err(mpsc::RecvTimeoutError::Disconnected) => {
            panic!("simulation worker exited without reporting completion (possible panic)");
        }
    }
}
