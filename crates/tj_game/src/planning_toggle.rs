// PlanningToggle: a button that enables planning mode and opens a
// commit/cancel modal. Headless-safe; tests drive it with `Interaction`.

use bevy::prelude::*;

pub const TOGGLE_LABEL: &str = "Enable Planning Mode";
/// Label shown while planning mode is on.
pub const TOGGLE_LABEL_ACTIVE: &str = "Disable Planning Mode";

/// Planning flags the toggle and its modal read and write.
#[derive(Resource, Default, Debug, Clone, Copy, PartialEq, Eq)]
pub struct PlanningToggleState {
    /// Planning mode is on. Cancel keeps it on; only the player turns it off.
    pub planning: bool,
    /// The commit/cancel modal is visible.
    pub modal_open: bool,
    /// The player committed the current plan.
    pub committed: bool,
}

#[derive(Component)]
pub struct PlanningToggleButton;

/// Marks the button's text so the label can be kept in sync with state.
#[derive(Component)]
pub struct PlanningToggleLabel;

#[derive(Component)]
pub struct PlanningModal;

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlanningModalAction {
    Commit,
    Cancel,
}

pub struct PlanningTogglePlugin;

impl Plugin for PlanningTogglePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PlanningToggleState>()
            .add_systems(Startup, spawn_planning_toggle)
            .add_systems(
                Update,
                (
                    planning_toggle_interactions,
                    sync_planning_modal,
                    sync_planning_toggle_label,
                )
                    .chain(),
            );
    }
}

fn text(value: &str) -> TextBundle {
    TextBundle::from_section(
        value,
        TextStyle {
            font_size: 16.0,
            color: Color::WHITE,
            ..default()
        },
    )
}

fn spawn_planning_toggle(mut commands: Commands) {
    commands
        .spawn(NodeBundle {
            style: Style {
                position_type: PositionType::Absolute,
                top: Val::Px(16.0),
                right: Val::Px(16.0),
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(8.0),
                ..default()
            },
            ..default()
        })
        .with_children(|root| {
            root.spawn((ButtonBundle::default(), PlanningToggleButton))
                .with_children(|button| {
                    button.spawn((text(TOGGLE_LABEL), PlanningToggleLabel));
                });
            root.spawn((
                NodeBundle {
                    style: Style {
                        display: Display::None,
                        column_gap: Val::Px(8.0),
                        ..default()
                    },
                    ..default()
                },
                PlanningModal,
            ))
            .with_children(|modal| {
                for (label, action) in [
                    ("Commit", PlanningModalAction::Commit),
                    ("Cancel", PlanningModalAction::Cancel),
                ] {
                    modal
                        .spawn((ButtonBundle::default(), action))
                        .with_children(|button| {
                            button.spawn(text(label));
                        });
                }
            });
        });
}

fn planning_toggle_interactions(
    mut state: ResMut<PlanningToggleState>,
    toggle: Query<&Interaction, (Changed<Interaction>, With<PlanningToggleButton>)>,
    actions: Query<(&Interaction, &PlanningModalAction), Changed<Interaction>>,
) {
    if toggle.iter().any(|i| *i == Interaction::Pressed) {
        state.planning = true;
        state.modal_open = true;
    }
    for (interaction, action) in &actions {
        if *interaction != Interaction::Pressed {
            continue;
        }
        state.modal_open = false;
        if *action == PlanningModalAction::Commit {
            state.committed = true;
        }
    }
}

/// Keeps the button label in sync with the planning state so the button
/// always offers the action the player can take next.
fn sync_planning_toggle_label(
    state: Res<PlanningToggleState>,
    mut labels: Query<&mut Text, With<PlanningToggleLabel>>,
) {
    if !state.is_changed() {
        return;
    }
    let desired = if state.planning {
        TOGGLE_LABEL_ACTIVE
    } else {
        TOGGLE_LABEL
    };
    for mut label in &mut labels {
        if label.sections[0].value != desired {
            label.sections[0].value = desired.to_string();
        }
    }
}

fn sync_planning_modal(
    state: Res<PlanningToggleState>,
    mut modals: Query<&mut Style, With<PlanningModal>>,
) {
    for mut style in &mut modals {
        style.display = if state.modal_open {
            Display::Flex
        } else {
            Display::None
        };
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .add_plugins(PlanningTogglePlugin);
        app.update();
        app
    }

    fn press<F: bevy::ecs::query::QueryFilter>(app: &mut App) {
        let entity = app.world.query_filtered::<Entity, F>().single(&app.world);
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
        *app.world.get_mut::<Interaction>(entity).unwrap() = Interaction::Pressed;
        app.update();
    }

    fn modal_display(app: &mut App) -> Display {
        app.world
            .query_filtered::<&Style, With<PlanningModal>>()
            .single(&app.world)
            .display
    }

    fn label_of(app: &App, parent: Entity) -> String {
        let children = app.world.get::<Children>(parent).unwrap();
        app.world.get::<Text>(children[0]).unwrap().sections[0]
            .value
            .clone()
    }

    fn state(app: &App) -> PlanningToggleState {
        *app.world.resource::<PlanningToggleState>()
    }

    #[test]
    fn button_renders_with_enable_planning_mode_label() {
        let mut app = app();
        let button = app
            .world
            .query_filtered::<Entity, With<PlanningToggleButton>>()
            .single(&app.world);
        assert_eq!(label_of(&app, button), "Enable Planning Mode");
        assert_eq!(modal_display(&mut app), Display::None);
    }

    #[test]
    fn button_label_reflects_current_planning_state() {
        let mut app = app();
        let button = app
            .world
            .query_filtered::<Entity, With<PlanningToggleButton>>()
            .single(&app.world);
        assert_eq!(label_of(&app, button), "Enable Planning Mode");
        press::<With<PlanningToggleButton>>(&mut app);
        assert_eq!(label_of(&app, button), "Disable Planning Mode");
        // Cancel keeps planning mode on, so the label stays active.
        press_action(&mut app, PlanningModalAction::Cancel);
        assert_eq!(label_of(&app, button), "Disable Planning Mode");
        press_action(&mut app, PlanningModalAction::Commit);
        assert_eq!(label_of(&app, button), "Disable Planning Mode");
    }

    #[test]
    fn clicking_button_displays_modal() {
        let mut app = app();
        press::<With<PlanningToggleButton>>(&mut app);
        assert_eq!(modal_display(&mut app), Display::Flex);
        assert!(state(&app).planning);
        assert!(state(&app).modal_open);
    }

    #[test]
    fn modal_shows_commit_and_cancel_buttons() {
        let mut app = app();
        let buttons: Vec<(Entity, PlanningModalAction)> = app
            .world
            .query::<(Entity, &PlanningModalAction)>()
            .iter(&app.world)
            .map(|(e, a)| (e, *a))
            .collect();
        let mut labels: Vec<(PlanningModalAction, String)> = buttons
            .into_iter()
            .map(|(e, a)| (a, label_of(&app, e)))
            .collect();
        labels.sort_by_key(|(_, l)| l.clone());
        assert_eq!(
            labels,
            [
                (PlanningModalAction::Cancel, "Cancel".to_string()),
                (PlanningModalAction::Commit, "Commit".to_string()),
            ]
        );
        let modal = app
            .world
            .query_filtered::<Entity, With<PlanningModal>>()
            .single(&app.world);
        assert_eq!(app.world.get::<Children>(modal).unwrap().len(), 2);
    }

    #[test]
    fn cancel_hides_modal_and_keeps_planning_state() {
        let mut app = app();
        press::<With<PlanningToggleButton>>(&mut app);
        press_action(&mut app, PlanningModalAction::Cancel);
        assert_eq!(modal_display(&mut app), Display::None);
        assert_eq!(
            state(&app),
            PlanningToggleState {
                planning: true,
                modal_open: false,
                committed: false,
            }
        );
    }

    #[test]
    fn commit_hides_modal_and_sets_committed_flag() {
        let mut app = app();
        press::<With<PlanningToggleButton>>(&mut app);
        press_action(&mut app, PlanningModalAction::Commit);
        assert_eq!(modal_display(&mut app), Display::None);
        assert_eq!(
            state(&app),
            PlanningToggleState {
                planning: true,
                modal_open: false,
                committed: true,
            }
        );
    }
}
