// Integration check: the PlanningToggle sits in the composed main UI
// alongside the other on-screen widgets without interfering with them.
//
// All UI plugins are mounted together, as in main.rs. Bevy UI positions are
// resolved by Taffy at render time, so this exercises the structural layout
// contract that matters: the toggle is absolutely anchored to the top-right
// corner (so it stays visible and readable on any window/viewport size),
// its modal is hidden until the toggle is pressed, and the widgets that
// share the screen (moving-vehicle counter top-left, plan comparison panel
// top-right below the toggle row) keep their non-overlapping anchors and
// keep rendering while the modal is open.

use bevy::prelude::*;
use tj_game::planning_toggle::{
    PlanningModal, PlanningModalAction, PlanningToggleButton, PlanningTogglePlugin,
    PlanningToggleState, TOGGLE_LABEL,
};
use tj_game::{
    ComparisonPanel, MovingVehicle, MovingVehicleCountLabel, MovingVehicleCountUiPlugin,
    PlanComparisonPanelPlugin, PopulationConsolePlugin,
};

fn composed_ui() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_plugins(PopulationConsolePlugin)
        .add_plugins(MovingVehicleCountUiPlugin)
        .add_plugins(PlanComparisonPanelPlugin)
        .add_plugins(PlanningTogglePlugin);
    app.update(); // Startup: every widget spawns.
    app
}

fn entity_with<T: Component>(app: &mut App) -> Entity {
    app.world
        .query_filtered::<Entity, With<T>>()
        .single(&app.world)
}

fn press(app: &mut App, entity: Entity) {
    *app.world.get_mut::<Interaction>(entity).unwrap() = Interaction::Pressed;
    app.update();
}

fn press_action(app: &mut App, action: PlanningModalAction) {
    let entity = app
        .world
        .query::<(Entity, &PlanningModalAction)>()
        .iter(&app.world)
        .find(|(_, a)| **a == action)
        .map(|(e, _)| e)
        .unwrap();
    press(app, entity);
}

fn style_of<T: Component>(app: &mut App) -> Style {
    let entity = entity_with::<T>(app);
    app.world.get::<Style>(entity).unwrap().clone()
}

fn root_style_of<T: Component>(app: &mut App) -> Style {
    let entity = entity_with::<T>(app);
    let root = app.world.get::<Parent>(entity).unwrap().get();
    app.world.get::<Style>(root).unwrap().clone()
}

#[test]
fn toggle_is_installed_once_in_the_composed_main_ui() {
    let mut app = composed_ui();
    let toggles = app
        .world
        .query_filtered::<Entity, With<PlanningToggleButton>>()
        .iter(&app.world)
        .count();
    assert_eq!(toggles, 1, "exactly one planning toggle in the final UI");

    // Absolutely anchored top-right: visible at every viewport size.
    let style = style_of::<PlanningToggleButton>(&mut app);
    let root_style = root_style_of::<PlanningToggleButton>(&mut app);
    assert_eq!(root_style.position_type, PositionType::Absolute);
    assert_eq!(root_style.top, Val::Px(16.0));
    assert_eq!(root_style.right, Val::Px(16.0));
    assert_eq!(style.display, Display::Flex, "button starts visible");
    assert_eq!(style.min_width, Val::Px(196.0));
    let button = entity_with::<PlanningToggleButton>(&mut app);
    assert_eq!(
        app.world.get::<BackgroundColor>(button).unwrap().0,
        Color::rgb(0.12, 0.32, 0.48),
        "the visible toggle has a contrasting button background"
    );

    // The modal starts hidden.
    assert_eq!(style_of::<PlanningModal>(&mut app).display, Display::None);
}

#[test]
fn existing_widgets_keep_their_anchors_and_render_with_the_toggle_present() {
    let mut app = composed_ui();

    // Counter is top-left, toggle column is top-right, plan panel is pinned
    // below the toggle row on the right edge: no two widgets share an anchor.
    let counter_style = root_style_of::<MovingVehicleCountLabel>(&mut app);
    assert_eq!(counter_style.position_type, PositionType::Absolute);
    assert_eq!(counter_style.top, Val::Px(12.0));
    assert_eq!(counter_style.left, Val::Px(12.0));

    let panel_style = style_of::<ComparisonPanel>(&mut app);
    assert_eq!(panel_style.position_type, PositionType::Absolute);
    assert_eq!(panel_style.right, Val::Px(16.0));
    assert!(
        matches!(panel_style.top, Val::Px(v) if v > 80.0),
        "plan panel must sit below the toggle row: {:?}",
        panel_style.top
    );

    // The counter still tracks movement with the toggle mounted.
    app.world.spawn(MovingVehicle);
    app.update();
    let label_entity = entity_with::<MovingVehicleCountLabel>(&mut app);
    let label = app.world.get::<Text>(label_entity).unwrap();
    assert_eq!(label.sections[0].value, "Moving vehicles: 1");
}

#[test]
fn modal_interaction_in_the_composed_ui_matches_plugin_contract() {
    let mut app = composed_ui();

    let toggle = entity_with::<PlanningToggleButton>(&mut app);
    press(&mut app, toggle);
    assert!(app.world.resource::<PlanningToggleState>().modal_open);
    assert_eq!(style_of::<PlanningModal>(&mut app).display, Display::Flex);

    // While the modal is open, the other widgets remain mounted and their
    // styles untouched (no interference).
    assert_eq!(
        style_of::<ComparisonPanel>(&mut app).display,
        Display::None,
        "hidden panel stays hidden while modal opens"
    );

    press_action(&mut app, PlanningModalAction::Commit);
    let state = *app.world.resource::<PlanningToggleState>();
    assert!(state.planning && state.committed && !state.modal_open);
    assert_eq!(style_of::<PlanningModal>(&mut app).display, Display::None);
}

#[test]
fn toggle_label_uses_the_shared_constants() {
    let mut app = composed_ui();
    let button = entity_with::<PlanningToggleButton>(&mut app);
    let text_child = app.world.get::<Children>(button).unwrap()[0];
    let label = app.world.get::<Text>(text_child).unwrap();
    assert_eq!(label.sections[0].value, TOGGLE_LABEL);
}
