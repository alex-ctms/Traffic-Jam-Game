use bevy::prelude::*;

fn main() {
    App::new()
        .add_plugins(tj_game::PopulationConsolePlugin)
        .add_plugins(tj_game::MovingVehicleCountUiPlugin)
        .add_plugins(tj_game::PlanComparisonPanelPlugin)
        .insert_resource(Monitor)
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Traffic Jam Game".to_string(),
                canvas: Some("#canvas".to_string()),
                ..default()
            }),
            ..default()
        }))
        .run();
}

#[derive(Resource)]
struct Monitor;
