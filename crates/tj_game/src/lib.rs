// Game module: headless-safe Bevy app wiring.
//
// Bevy 0.13 has no `add_startup_system` method. That name is a Bevy 0.14+ API.
// The 0.13 way is `add_systems(Startup, ...)`. Keeping this note because the
// mismatch is silent until compile time.

use bevy::prelude::*;
use tj_population::PopulationStats;

/// Aggregate counts available to developer console commands.
#[derive(Resource, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct CityPopulation(pub PopulationStats);

/// Run a developer console command against the current city state.
///
/// Returns `None` for commands not handled by this module.
pub fn run_console_command(command: &str, population: PopulationStats) -> Option<String> {
    match command.trim() {
        "/pop_stats" => Some(population.format_console_output()),
        _ => None,
    }
}

/// Register the population stats command with a Bevy app.
pub struct PopulationConsolePlugin;

impl Plugin for PopulationConsolePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<CityPopulation>()
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

/// Marks a frontend vehicle entity that is currently moving.
///
/// Vehicle presentation code should add or remove this component as the
/// corresponding simulation vehicle starts or stops moving.
#[derive(Component, Clone, Copy, Debug, Default)]
pub struct MovingVehicle;

/// Current number of moving frontend vehicles.
#[derive(Resource, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct MovingVehicleCount(pub usize);

#[derive(Component)]
struct MovingVehicleCountLabel;

/// Displays the number of ECS vehicle entities currently marked as moving.
pub struct MovingVehicleCountUiPlugin;

impl Plugin for MovingVehicleCountUiPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<MovingVehicleCount>()
            .add_systems(Startup, setup_moving_vehicle_count_ui)
            .add_systems(Update, update_moving_vehicle_count);
    }
}

fn setup_moving_vehicle_count_ui(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
    commands
        .spawn(NodeBundle {
            style: Style {
                position_type: PositionType::Absolute,
                top: Val::Px(12.0),
                left: Val::Px(12.0),
                padding: UiRect::all(Val::Px(8.0)),
                ..default()
            },
            background_color: Color::rgba(0.05, 0.05, 0.05, 0.8).into(),
            ..default()
        })
        .with_children(|parent| {
            parent.spawn((
                TextBundle::from_section(
                    "Moving vehicles: 0",
                    TextStyle {
                        font_size: 20.0,
                        color: Color::WHITE,
                        ..default()
                    },
                ),
                MovingVehicleCountLabel,
            ));
        });
}

fn update_moving_vehicle_count(
    vehicles: Query<(), With<MovingVehicle>>,
    mut count: ResMut<MovingVehicleCount>,
    mut labels: Query<&mut Text, With<MovingVehicleCountLabel>>,
) {
    count.0 = vehicles.iter().count();
    for mut label in &mut labels {
        label.sections[0].value = format!("Moving vehicles: {}", count.0);
    }
}

fn process_console_commands(
    mut commands: ResMut<ConsoleCommands>,
    population: Res<CityPopulation>,
    mut output: ResMut<ConsoleOutput>,
) {
    for command in commands.0.drain(..) {
        if let Some(result) = run_console_command(&command, population.0) {
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
}

#[cfg(test)]
mod moving_vehicle_ui_tests {
    use super::*;

    #[test]
    fn widget_count_tracks_moving_vehicle_entities() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .add_plugins(MovingVehicleCountUiPlugin);
        app.world.spawn(MovingVehicle);
        app.world.spawn(MovingVehicle);
        app.world.spawn_empty(); // stationary/non-vehicle entities are excluded

        app.update();

        assert_eq!(app.world.resource::<MovingVehicleCount>().0, 2);
        let mut labels = app
            .world
            .query_filtered::<&Text, With<MovingVehicleCountLabel>>();
        assert_eq!(
            labels.single(&app.world).sections[0].value,
            "Moving vehicles: 2"
        );
    }
}
