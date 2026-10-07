// Game module: headless-safe Bevy app wiring.
//
// Bevy 0.13 has no `add_startup_system` method. That name is a Bevy 0.14+ API.
// The 0.13 way is `add_systems(Startup, ...)`. Keeping this note because the
// mismatch is silent until compile time.

use bevy::prelude::*;
use tj_network::{LaneGraph, ZoneType};
use tj_population::PopulationStats;

/// Aggregate counts available to developer console commands.
#[derive(Resource, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct CityPopulation(pub PopulationStats);

/// Road graph available to developer console commands.
#[derive(Resource, Default)]
pub struct RoadNetwork(pub LaneGraph);

/// Run a developer console command against the current city state.
///
/// Returns `None` for commands not handled by this module.
pub fn run_console_command(command: &str, population: PopulationStats) -> Option<String> {
    match command.trim() {
        "/pop_stats" => Some(population.format_console_output()),
        _ => None,
    }
}

/// Create a two-point sample road and zone its frontage residential.
///
/// Returns the allocated road ID, or `None` if the graph cannot allocate a road.
pub fn create_default_residential_road(network: &mut LaneGraph) -> Option<u32> {
    let road_id = network.add_segment(1, 2, 10.0)?;
    let segment_id = network.get_segments(road_id)?.first()?.id;
    network
        .assign_zone(road_id, segment_id, ZoneType::Residential)
        .then_some(road_id)
}

/// Register the population stats command with a Bevy app.
pub struct PopulationConsolePlugin;

impl Plugin for PopulationConsolePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<CityPopulation>()
            .init_resource::<RoadNetwork>()
            .init_resource::<ConsoleCommands>()
            .init_resource::<ConsoleOutput>()
            .add_systems(Update, process_console_commands);
    }
}

/// Commands submitted by the developer console frontend.
#[derive(Resource, Default)]
pub struct ConsoleCommands(pub Vec<String>);

/// Output lines produced by the console command handler.
#[derive(Resource, Default)]
pub struct ConsoleOutput(pub Vec<String>);

fn process_console_commands(
    mut commands: ResMut<ConsoleCommands>,
    population: Res<CityPopulation>,
    mut network: ResMut<RoadNetwork>,
    mut output: ResMut<ConsoleOutput>,
) {
    for command in commands.0.drain(..) {
        if let Some(result) = run_console_command(&command, population.0) {
            println!("{result}");
            output.0.push(result);
        } else if command.trim() == "/create_road" {
            let result = create_default_residential_road(&mut network.0)
                .map(|id| format!("Created residential road {id} from point 1 to point 2"))
                .unwrap_or_else(|| "Failed to create residential road".to_string());
            println!("{result}");
            output.0.push(result);
        }
    }
}

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

#[cfg(test)]
mod console_tests {
    use super::*;

    #[test]
    fn pop_stats_command_prints_current_counts() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .add_plugins(PopulationConsolePlugin);
        app.world.resource_mut::<CityPopulation>().0 = PopulationStats {
            population: 1250,
            jobs: 840,
        };
        app.world
            .resource_mut::<ConsoleCommands>()
            .0
            .push("/pop_stats".to_string());

        app.update();

        assert_eq!(
            app.world.resource::<ConsoleOutput>().0,
            ["Population: 1250\nJobs: 840"]
        );
    }

    #[test]
    fn create_road_command_adds_two_point_residential_road() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .add_plugins(PopulationConsolePlugin);
        app.world
            .resource_mut::<ConsoleCommands>()
            .0
            .push("/create_road".to_string());

        app.update();

        let network = &app.world.resource::<RoadNetwork>().0;
        assert_eq!(network.num_roads(), 1);
        let segments = network.get_segments(0).expect("created road should exist");
        assert_eq!(segments.len(), 1);
        assert_eq!((segments[0].start_id, segments[0].end_id), (1, 2));
        assert_eq!(segments[0].length, 10.0);
        assert_eq!(segments[0].zone, Some(ZoneType::Residential));
        assert!(app.world.resource::<ConsoleOutput>().0[0].contains("residential road 0"));
    }
}
