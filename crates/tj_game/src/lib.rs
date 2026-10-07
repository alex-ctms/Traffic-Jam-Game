// Game module: headless-safe Bevy app wiring.
//
// Bevy 0.13 has no `add_startup_system` method. That name is a Bevy 0.14+ API.
// The 0.13 way is `add_systems(Startup, ...)`. Keeping this note because the
// mismatch is silent until compile time.

use bevy::prelude::*;
use tj_analytics::HeatmapData;
use tj_network::{LaneGraph, ZoneType};
use tj_population::PopulationStats;

/// Aggregate counts available to developer console commands.
#[derive(Resource, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct CityPopulation(pub PopulationStats);

/// Road graph available to developer console commands.
#[derive(Resource, Default)]
pub struct RoadNetwork(pub LaneGraph);

/// Heatmap shown by the congestion overlay. Set `data` from current simulation
/// analytics and toggle `visible` from the frontend to show or hide it.
#[derive(Resource, Default)]
pub struct CongestionHeatmapOverlay {
    pub data: Option<HeatmapData>,
    pub visible: bool,
}

#[derive(Component)]
struct CongestionHeatmapCell;

/// Renders analytics heatmap cells as translucent colored sprites.
pub struct CongestionHeatmapOverlayPlugin;

impl Plugin for CongestionHeatmapOverlayPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<CongestionHeatmapOverlay>()
            .add_systems(Update, update_congestion_heatmap_overlay);
    }
}

fn update_congestion_heatmap_overlay(
    mut commands: Commands,
    overlay: Res<CongestionHeatmapOverlay>,
    cells: Query<Entity, With<CongestionHeatmapCell>>,
) {
    if !overlay.is_changed() {
        return;
    }

    for entity in &cells {
        commands.entity(entity).despawn();
    }

    let Some(heatmap) = overlay.data.as_ref().filter(|_| overlay.visible) else {
        return;
    };
    let max_density = heatmap.data.iter().copied().max().unwrap_or(0);
    if max_density == 0 || heatmap.cell_size <= 0.0 {
        return;
    }

    for y in 0..heatmap.height {
        for x in 0..heatmap.width {
            let density = heatmap.get_density(x, y);
            if density == 0 {
                continue;
            }
            let intensity = density as f32 / max_density as f32;
            let color = congestion_heat_color(intensity);
            commands.spawn((
                SpriteBundle {
                    sprite: Sprite {
                        color,
                        custom_size: Some(Vec2::splat(heatmap.cell_size)),
                        ..default()
                    },
                    transform: Transform::from_xyz(
                        (x as f32 + 0.5) * heatmap.cell_size,
                        (y as f32 + 0.5) * heatmap.cell_size,
                        10.0,
                    ),
                    ..default()
                },
                CongestionHeatmapCell,
            ));
        }
    }
}

/// Maps normalized congestion to a blue → yellow → red translucent gradient.
fn congestion_heat_color(intensity: f32) -> Color {
    let t = intensity.clamp(0.0, 1.0);
    let (r, g, b) = if t < 0.5 {
        let blend = t * 2.0;
        (blend, 0.35 + 0.55 * blend, 1.0 - blend)
    } else {
        let blend = (t - 0.5) * 2.0;
        (1.0, 0.9 * (1.0 - blend), 0.0)
    };
    Color::rgba(r, g, b, 0.55)
}

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
        .add_plugins(CongestionHeatmapOverlayPlugin)
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

    #[test]
    fn approving_network_change_updates_zones_and_traffic_path() {
        let mut network = LaneGraph::new();
        let existing_road = network.add_segment(1, 2, 5.0).unwrap();
        network.assign_zone(
            existing_road,
            network.get_segments(existing_road).unwrap()[0].id,
            ZoneType::Residential,
        );

        // A planned extension is kept separate until approval.
        let planned_road = (2, 3, 5.0);
        assert!(network.get_lanes_at_intersection(3).is_none());

        // Approval applies the road and its frontage zoning to the live graph.
        let approved_road = network
            .add_segment(planned_road.0, planned_road.1, planned_road.2)
            .expect("approved road should be added");
        let segment_id = network.get_segments(approved_road).unwrap()[0].id;
        assert!(network.assign_zone(approved_road, segment_id, ZoneType::Commercial));

        assert_eq!(network.num_roads(), 2);
        assert_eq!(
            network.get_zone(approved_road, segment_id),
            Some(ZoneType::Commercial)
        );
        assert!(network.get_lanes_at_intersection(3).is_some());
        assert!(network.check_invariants());

        let first_lane = network
            .get_road_lanes(existing_road)
            .unwrap()
            .iter()
            .find(|lane| lane.start_id == 1 && lane.end_id == 2)
            .unwrap()
            .id;
        let second_lane = network
            .get_road_lanes(approved_road)
            .unwrap()
            .iter()
            .find(|lane| lane.start_id == 2 && lane.end_id == 3)
            .unwrap()
            .id;
        let mut traffic = tj_traffic::TrafficSimulation::new(network);
        let vehicle_id = traffic
            .spawn_vehicle_on_route(&[first_lane, second_lane], 5.0)
            .expect("route should reflect the approved connection");
        traffic.tick();
        assert_eq!(traffic.vehicle(vehicle_id).unwrap().lane_id, second_lane);
    }

    #[test]
    fn congestion_overlay_renders_density_cells_and_hides_on_toggle() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .add_plugins(CongestionHeatmapOverlayPlugin);
        app.update();

        let mut heatmap = HeatmapData::new(20.0, 2, 1);
        heatmap.set_density(0, 0, 1);
        heatmap.set_density(1, 0, 4);
        {
            let mut overlay = app.world.resource_mut::<CongestionHeatmapOverlay>();
            overlay.data = Some(heatmap);
            overlay.visible = true;
        }
        app.update();

        let rendered: Vec<_> = app
            .world
            .query_filtered::<(&Sprite, &Transform), With<CongestionHeatmapCell>>()
            .iter(&app.world)
            .collect();
        assert_eq!(rendered.len(), 2);
        assert_eq!(rendered[0].0.custom_size, Some(Vec2::splat(20.0)));
        assert_eq!(rendered[1].1.translation.x, 30.0);
        assert_ne!(rendered[0].0.color, rendered[1].0.color);

        app.world.resource_mut::<CongestionHeatmapOverlay>().visible = false;
        app.update();
        assert_eq!(
            app.world
                .query_filtered::<Entity, With<CongestionHeatmapCell>>()
                .iter(&app.world)
                .count(),
            0
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
