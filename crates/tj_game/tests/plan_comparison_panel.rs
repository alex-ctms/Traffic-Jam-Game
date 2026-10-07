//! Headless check for the M6.4 plan comparison side-panel.

use bevy::prelude::{App, Display, MinimalPlugins, Style, Text, Update};
use tj_game::{format_comparison, PlanComparisonPanelPlugin, PlanComparisonState, PlanMetrics};

#[test]
fn comparison_lines_show_current_projected_and_delta_for_each_metric() {
    let current = PlanMetrics {
        average_commute_minutes: 24.0,
        congestion_percent: 61.5,
        cost_dollars: 1_200_000,
    };
    let projected = PlanMetrics {
        average_commute_minutes: 21.5,
        congestion_percent: 48.0,
        cost_dollars: 1_350_000,
    };

    let lines = format_comparison(current, projected);

    assert_eq!(lines.len(), 3);
    assert_eq!(
        lines[0], "Avg commute: 24.0 -> 21.5 min (-2.5)",
        "average commute must contrast current vs projected numerically"
    );
    assert_eq!(
        lines[1], "Congestion: 61.5% -> 48.0% (-13.5)",
        "congestion must contrast current vs projected numerically"
    );
    assert_eq!(
        lines[2], "Cost: $1200000 -> $1350000 (+150000)",
        "cost must contrast current vs projected numerically"
    );
}

#[test]
fn panel_stays_hidden_until_a_plan_is_applied_then_renders_comparison() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_plugins(PlanComparisonPanelPlugin);
    app.update(); // Startup: panel spawned hidden.

    use tj_game::ComparisonPanel;
    let mut panel_query = app
        .world
        .query_filtered::<&Style, bevy::prelude::With<ComparisonPanel>>();
    let style = panel_query.single(&app.world);
    assert_eq!(style.display, Display::None, "no plan applied yet");

    // A sibling system applies a plan and records the comparison.
    app.add_systems(Update, apply_one_plan_once);
    app.update();
    app.update();

    let style = panel_query.single(&app.world);
    assert_eq!(
        style.display,
        Display::Flex,
        "panel must become visible after a plan is applied"
    );

    use tj_game::ComparisonMetricLine;
    let mut line_query = app
        .world
        .query_filtered::<&Text, bevy::prelude::With<ComparisonMetricLine>>();
    let mut texts = line_query
        .iter(&app.world)
        .map(|text| text.sections[0].value.clone())
        .collect::<Vec<_>>();
    texts.sort();

    assert!(
        texts.contains(&"Avg commute: 30.0 -> 25.0 min (-5.0)".to_string()),
        "panel must show the commute comparison, got {texts:?}"
    );
    assert!(
        texts.contains(&"Congestion: 70.0% -> 55.0% (-15.0)".to_string()),
        "panel must show the congestion comparison, got {texts:?}"
    );
    assert!(
        texts.contains(&"Cost: $500000 -> $800000 (+300000)".to_string()),
        "panel must show the cost comparison, got {texts:?}"
    );
}

fn apply_one_plan_once(mut comparison: ResMut<PlanComparisonState>) {
    use tj_game::PlanMetrics;
    comparison.record_plan_application(
        PlanMetrics {
            average_commute_minutes: 30.0,
            congestion_percent: 70.0,
            cost_dollars: 500_000,
        },
        PlanMetrics {
            average_commute_minutes: 25.0,
            congestion_percent: 55.0,
            cost_dollars: 800_000,
        },
    );
}

use bevy::prelude::ResMut;
