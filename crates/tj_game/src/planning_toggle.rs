// PlanningToggle: a button that enables planning mode and opens a
// commit/cancel modal. Headless-safe; tests drive it with `Interaction`.

use bevy::{a11y::AccessibilityNode, prelude::*};

pub const TOGGLE_LABEL: &str = "Enable Planning Mode";
pub const PLANNING_MODAL_TITLE_ID: &str = "planning-modal-title";

/// Planning flags the toggle and its modal read and write.
#[derive(Resource, Default, Debug, Clone, Copy, PartialEq, Eq)]
pub struct PlanningToggleState {
    /// Planning mode is on. Cancel keeps it on; only the player turns it off.
    pub planning: bool,
    /// The commit/cancel modal is visible.
    pub modal_open: bool,
    /// The player committed the current plan.
    pub has_committed: bool,
}

impl PlanningToggleState {
    /// Commit button: record the commit and close the modal.
    pub fn handle_commit(&mut self) {
        self.has_committed = true;
        self.modal_open = false;
    }

    /// Cancel button: close the modal only; planning mode is left as is.
    pub fn handle_cancel(&mut self) {
        self.modal_open = false;
    }
}

#[derive(Component)]
pub struct PlanningToggleButton;

/// Marks the button's text so the label can be kept in sync with state.
#[derive(Component)]
pub struct PlanningToggleLabel;

#[derive(Component)]
pub struct PlanningModal;

/// Marks the visible title that names the planning modal.
#[derive(Component)]
pub struct PlanningModalTitle;

/// Stable identifier for the visible planning modal title.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct PlanningModalTitleId(pub &'static str);

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlanningModalAction {
    Commit,
    Cancel,
}

pub struct PlanningTogglePlugin;

impl Plugin for PlanningTogglePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PlanningToggleState>()
            .add_systems(
                Startup,
                (spawn_planning_toggle, link_planning_modal_title).chain(),
            )
            .add_systems(
                Update,
                (planning_toggle_interactions, sync_planning_modal).chain(),
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
            root.spawn((
                ButtonBundle {
                    style: Style {
                        min_width: Val::Px(196.0),
                        padding: UiRect::axes(Val::Px(12.0), Val::Px(8.0)),
                        justify_content: JustifyContent::Center,
                        ..default()
                    },
                    background_color: Color::rgb(0.12, 0.32, 0.48).into(),
                    ..default()
                },
                PlanningToggleButton,
            ))
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
                AccessibilityNode::from({
                    let mut node = bevy::a11y::accesskit::NodeBuilder::new(
                        bevy::a11y::accesskit::Role::Dialog,
                    );
                    node.set_name("Planning Mode".to_string());
                    node
                }),
                PlanningModal,
            ))
            .with_children(|modal| {
                modal.spawn((
                    text("Planning Mode"),
                    PlanningModalTitle,
                    PlanningModalTitleId(PLANNING_MODAL_TITLE_ID),
                    AccessibilityNode::from({
                        let mut node = bevy::a11y::accesskit::NodeBuilder::new(
                            bevy::a11y::accesskit::Role::Heading,
                        );
                        node.set_name("Planning Mode".to_string());
                        node
                    }),
                ));
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

fn link_planning_modal_title(
    mut modals: Query<&mut AccessibilityNode, With<PlanningModal>>,
    titles: Query<Entity, With<PlanningModalTitle>>,
) {
    let Ok(title) = titles.get_single() else {
        return;
    };
    for mut modal in &mut modals {
        modal.set_labelled_by(vec![bevy::a11y::accesskit::NodeId(title.to_bits())]);
    }
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
        match action {
            PlanningModalAction::Commit => state.handle_commit(),
            PlanningModalAction::Cancel => state.handle_cancel(),
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
    fn toggle_keeps_its_required_accessible_label_after_opening_modal() {
        let mut app = app();
        let button = app
            .world
            .query_filtered::<Entity, With<PlanningToggleButton>>()
            .single(&app.world);
        assert_eq!(label_of(&app, button), "Enable Planning Mode");
        press::<With<PlanningToggleButton>>(&mut app);
        assert_eq!(label_of(&app, button), "Enable Planning Mode");
        press_action(&mut app, PlanningModalAction::Cancel);
        assert_eq!(label_of(&app, button), "Enable Planning Mode");
        assert!(!state(&app).has_committed);
        press_action(&mut app, PlanningModalAction::Commit);
        assert_eq!(label_of(&app, button), "Enable Planning Mode");
        assert!(state(&app).has_committed);
        assert_eq!(modal_display(&mut app), Display::None);
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
        assert_eq!(app.world.get::<Children>(modal).unwrap().len(), 3);
    }

    #[test]
    fn modal_has_a_visible_title_and_matching_accessible_name() {
        let mut app = app();
        let modal = app
            .world
            .query_filtered::<Entity, With<PlanningModal>>()
            .single(&app.world);
        let title = app
            .world
            .query_filtered::<Entity, With<PlanningModalTitle>>()
            .single(&app.world);
        assert!(app.world.get::<Children>(modal).unwrap().contains(&title));
        assert_eq!(
            app.world.get::<PlanningModalTitleId>(title).unwrap().0,
            PLANNING_MODAL_TITLE_ID
        );

        assert_eq!(
            app.world.get::<Text>(title).unwrap().sections[0].value,
            "Planning Mode"
        );
        assert_eq!(
            app.world.get::<AccessibilityNode>(modal).unwrap().name(),
            Some("Planning Mode")
        );
        assert_eq!(
            app.world
                .get::<AccessibilityNode>(modal)
                .unwrap()
                .labelled_by(),
            &[bevy::a11y::accesskit::NodeId(title.to_bits())]
        );
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
                has_committed: false,
            }
        );
    }

    #[test]
    fn has_committed_flag_starts_false() {
        let app = app();
        assert!(!state(&app).has_committed);
    }

    #[test]
    fn handle_commit_sets_has_committed_and_closes_modal() {
        let mut s = PlanningToggleState {
            planning: true,
            modal_open: true,
            has_committed: false,
        };
        s.handle_commit();
        assert_eq!(
            s,
            PlanningToggleState {
                planning: true,
                modal_open: false,
                has_committed: true,
            }
        );
    }

    #[test]
    fn handle_cancel_only_closes_modal() {
        for has_committed in [false, true] {
            let mut s = PlanningToggleState {
                planning: true,
                modal_open: true,
                has_committed,
            };
            s.handle_cancel();
            assert_eq!(
                s,
                PlanningToggleState {
                    planning: true,
                    modal_open: false,
                    has_committed,
                }
            );
        }
    }

    #[test]
    fn commit_hides_modal_and_sets_has_committed_flag() {
        let mut app = app();
        press::<With<PlanningToggleButton>>(&mut app);
        press_action(&mut app, PlanningModalAction::Commit);
        assert_eq!(modal_display(&mut app), Display::None);
        assert_eq!(
            state(&app),
            PlanningToggleState {
                planning: true,
                modal_open: false,
                has_committed: true,
            }
        );
    }
}
