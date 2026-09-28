// Game module placeholder

pub fn run() {
    use bevy::prelude::*;
    App::new()
        .add_plugins(DefaultPlugins)
        .add_startup_system(setup)
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
    // Placeholder: will hide window in CI if needed
}

