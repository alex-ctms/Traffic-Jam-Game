// Game module: headless-safe Bevy app wiring.
//
// Bevy 0.13 has no `add_startup_system` method. That name is a Bevy 0.14+ API.
// The 0.13 way is `add_systems(Startup, ...)`. Keeping this note because the
// mismatch is silent until compile time.

use bevy::prelude::*;

/// Counts trips produced by the simulation and retained in its pending queue.
#[derive(Resource, Default)]
pub struct TripQueue {
    pub generated: u64,
    pub trips: Vec<Trip>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Trip {
    pub origin: u64,
    pub destination: u64,
}

/// Headless-safe demand generation used by the game and simulation tests.
pub struct DemandPlugin;

impl Plugin for DemandPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<TripQueue>()
            .add_systems(Update, generate_and_queue_trip);
    }
}

fn generate_and_queue_trip(mut queue: ResMut<TripQueue>) {
    let id = queue.generated;
    queue.trips.push(Trip {
        origin: id,
        destination: id + 1,
    });
    queue.generated += 1;
}

pub fn run() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, setup)
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
}
