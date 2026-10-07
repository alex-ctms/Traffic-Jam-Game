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

/// The three numeric metrics the plan comparison side-panel contrasts.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PlanMetrics {
    /// Average commute across the city in minutes.
    pub average_commute_minutes: f32,
    /// Citywide congestion level as a percentage (0-100).
    pub congestion_percent: f32,
    /// Cost in dollars.
    pub cost_dollars: i64,
}

/// State backing the plan comparison side-panel. Populated when a plan is
/// applied; the panel stays hidden until both sides of the comparison exist.
#[derive(Resource, Clone, Debug, Default)]
pub struct PlanComparisonState {
    pub baseline: Option<PlanMetrics>,
    pub projected: Option<PlanMetrics>,
}

impl PlanComparisonState {
    /// Capture the current metrics and the projection when a plan is applied.
    pub fn record_plan_application(&mut self, current: PlanMetrics, projected: PlanMetrics) {
        self.baseline = Some(current);
        self.projected = Some(projected);
    }
}

/// Build the panel lines: one per metric, showing current vs projected with
/// the signed delta. Positive deltas are increases, negative are reductions.
pub fn format_comparison(current: PlanMetrics, projected: PlanMetrics) -> Vec<String> {
    vec![
        format!(
            "Avg commute: {:.1} -> {:.1} min ({:+.1})",
            current.average_commute_minutes,
            projected.average_commute_minutes,
            projected.average_commute_minutes - current.average_commute_minutes
        ),
        format!(
            "Congestion: {:.1}% -> {:.1}% ({:+.1})",
            current.congestion_percent,
            projected.congestion_percent,
            projected.congestion_percent - current.congestion_percent
        ),
        format!(
            "Cost: ${} -> ${} ({:+})",
            current.cost_dollars,
            projected.cost_dollars,
            projected.cost_dollars - current.cost_dollars
        ),
    ]
}

#[derive(Component)]
pub struct ComparisonPanel;

/// Marks one metric line inside the comparison panel; the index selects
/// which formatted line (commute, congestion, cost) fills its text.
#[derive(Component)]
pub struct ComparisonMetricLine(pub usize);

/// Side-panel UI (docked right) contrasting current vs projected metrics
/// after a plan is applied. Other systems fill PlanComparisonState on
/// approval; this plugin only renders it.
pub struct PlanComparisonPanelPlugin;

impl Plugin for PlanComparisonPanelPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PlanComparisonState>()
            .add_systems(Startup, spawn_comparison_panel)
            .add_systems(Update, sync_comparison_panel);
    }
}

fn spawn_comparison_panel(mut commands: Commands) {
    commands
        .spawn((
            NodeBundle {
                style: Style {
                    position_type: PositionType::Absolute,
                    right: Val::Px(16.0),
                    top: Val::Px(104.0),
                    width: Val::Px(260.0),
                    flex_direction: FlexDirection::Column,
                    row_gap: Val::Px(4.0),
                    padding: UiRect::all(Val::Px(12.0)),
                    display: Display::None,
                    ..default()
                },
                background_color: BackgroundColor(Color::rgb(0.08, 0.10, 0.14)),
                ..default()
            },
            ComparisonPanel,
        ))
        .with_children(|panel| {
            panel.spawn(TextBundle::from_section(
                "Plan impact",
                TextStyle {
                    font_size: 18.0,
                    color: Color::WHITE,
                    ..default()
                },
            ));
            for index in 0..3 {
                panel.spawn((
                    TextBundle::from_section(
                        "",
                        TextStyle {
                            font_size: 14.0,
                            color: Color::rgb(0.8, 0.85, 0.9),
                            ..default()
                        },
                    ),
                    ComparisonMetricLine(index),
                ));
            }
        });
}

fn sync_comparison_panel(
    comparison: Res<PlanComparisonState>,
    mut panel: Query<&mut Style, With<ComparisonPanel>>,
    mut lines: Query<(&ComparisonMetricLine, &mut Text), With<ComparisonMetricLine>>,
) {
    let visible = matches!(
        (comparison.baseline, comparison.projected),
        (Some(_), Some(_))
    );
    for mut style in &mut panel {
        style.display = if visible {
            Display::Flex
        } else {
            Display::None
        };
    }
    if !visible {
        return;
    }
    let current = comparison.baseline.unwrap();
    let projected = comparison.projected.unwrap();
    let formatted = format_comparison(current, projected);
    for (line, mut text) in &mut lines {
        text.sections[0].value = formatted[line.0].clone();
    }
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
