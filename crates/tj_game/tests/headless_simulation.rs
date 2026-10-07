use bevy::prelude::{App, MinimalPlugins};
use tj_game::{DemandPlugin, TripQueue};

#[test]
fn ten_headless_ticks_queue_every_generated_trip_without_panicking() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins).add_plugins(DemandPlugin);

    for _ in 0..10 {
        app.update();
    }

    let queue = app.world.resource::<TripQueue>();
    assert_eq!(queue.generated, 10);
    assert_eq!(queue.trips.len() as u64, queue.generated);
    assert!(queue
        .trips
        .iter()
        .all(|trip| trip.origin != trip.destination));
}
