use bevy::prelude::*;

fn main() {
    App::new()
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