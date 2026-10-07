//! Isolated planning overlay for previewing network edits before approval.

use crate::{LaneGraph, ZoneType};

/// An edit staged in a [`PlanningOverlay`].
#[derive(Debug, Clone, PartialEq)]
pub enum PlannedChange {
    AddRoad {
        segments: Vec<(u32, u32, f32)>,
        road_id: u32,
    },
    RemoveRoad {
        road_id: u32,
    },
    AssignZone {
        road_id: u32,
        segment_id: u64,
        zone: ZoneType,
    },
}

/// A private working copy of the live network. Staged edits affect only this copy
/// until `approve_all_changes` successfully commits them to the owned live graph.
#[derive(Debug, Clone)]
pub struct PlanningOverlay {
    live_graph: LaneGraph,
    base_graph: LaneGraph,
    overlay_graph: LaneGraph,
    pending_changes: Vec<PlannedChange>,
    active: bool,
}

impl PlanningOverlay {
    pub fn new(live_graph: LaneGraph) -> Self {
        Self {
            base_graph: live_graph.clone(),
            overlay_graph: live_graph.clone(),
            live_graph,
            pending_changes: Vec::new(),
            active: false,
        }
    }

    pub fn enable(&mut self) {
        self.active = true;
    }

    /// Disable and discard the current plan.
    pub fn disable(&mut self) {
        self.active = false;
        self.discard_all_changes();
    }

    pub fn is_active(&self) -> bool {
        self.active
    }

    pub fn propose_add_road(&mut self, segments: Vec<(u32, u32, f32)>) -> Option<u32> {
        if !self.active {
            return None;
        }
        let road_id = self.overlay_graph.add_road(segments.clone())?;
        self.pending_changes
            .push(PlannedChange::AddRoad { segments, road_id });
        Some(road_id)
    }

    pub fn propose_remove_road(&mut self, road_id: u32) -> bool {
        if !self.active || !self.overlay_graph.remove_road(road_id) {
            return false;
        }
        self.pending_changes
            .push(PlannedChange::RemoveRoad { road_id });
        true
    }

    pub fn propose_zone_assignment(
        &mut self,
        road_id: u32,
        segment_id: u64,
        zone: ZoneType,
    ) -> bool {
        if !self.active || !self.overlay_graph.assign_zone(road_id, segment_id, zone) {
            return false;
        }
        self.pending_changes.push(PlannedChange::AssignZone {
            road_id,
            segment_id,
            zone,
        });
        true
    }

    /// Apply the full plan atomically. A changed live graph invalidates the plan.
    pub fn approve_all_changes(&mut self) -> Result<(), String> {
        if self.live_graph != self.base_graph {
            return Err("live graph changed since planning began".into());
        }
        let mut committed = self.live_graph.clone();
        for change in &self.pending_changes {
            match change {
                PlannedChange::AddRoad { segments, road_id } => {
                    if committed.add_road(segments.clone()) != Some(*road_id) {
                        return Err("planned road ID no longer matches live graph".into());
                    }
                }
                PlannedChange::RemoveRoad { road_id } => {
                    if !committed.remove_road(*road_id) {
                        return Err(format!("road {road_id} no longer exists"));
                    }
                }
                PlannedChange::AssignZone {
                    road_id,
                    segment_id,
                    zone,
                } => {
                    if !committed.assign_zone(*road_id, *segment_id, *zone) {
                        return Err("planned segment no longer exists".into());
                    }
                }
            }
        }
        self.live_graph = committed;
        self.reset_overlay();
        Ok(())
    }

    /// Commit one staged edit, preserving the remaining plan.
    pub fn approve_change(&mut self, change_index: usize) -> Result<(), String> {
        if self.live_graph != self.base_graph {
            return Err("live graph changed since planning began".into());
        }
        if change_index >= self.pending_changes.len() {
            return Err("Invalid change index".into());
        }
        let change = self.pending_changes.remove(change_index);
        let mut committed = self.live_graph.clone();
        let applied = match &change {
            PlannedChange::AddRoad { segments, road_id } => {
                committed.add_road(segments.clone()) == Some(*road_id)
            }
            PlannedChange::RemoveRoad { road_id } => committed.remove_road(*road_id),
            PlannedChange::AssignZone {
                road_id,
                segment_id,
                zone,
            } => committed.assign_zone(*road_id, *segment_id, *zone),
        };
        if !applied {
            self.pending_changes.insert(change_index, change);
            return Err("planned change cannot be applied to live graph".into());
        }
        self.live_graph = committed;
        // Rebuild remaining overlay by replaying its staged operations against the newly committed base.
        self.base_graph = self.live_graph.clone();
        self.rebuild_overlay()?;
        Ok(())
    }

    pub fn get_overlay_graph(&self) -> &LaneGraph {
        &self.overlay_graph
    }
    pub fn get_live_graph(&self) -> &LaneGraph {
        &self.live_graph
    }
    pub fn has_pending_changes(&self) -> bool {
        !self.pending_changes.is_empty()
    }
    pub fn pending_change_count(&self) -> usize {
        self.pending_changes.len()
    }
    pub fn pending_changes(&self) -> &[PlannedChange] {
        &self.pending_changes
    }

    pub fn discard_all_changes(&mut self) {
        self.pending_changes.clear();
        self.overlay_graph = self.live_graph.clone();
        self.base_graph = self.live_graph.clone();
    }

    fn reset_overlay(&mut self) {
        self.pending_changes.clear();
        self.base_graph = self.live_graph.clone();
        self.overlay_graph = self.live_graph.clone();
    }

    fn rebuild_overlay(&mut self) -> Result<(), String> {
        let mut graph = self.live_graph.clone();
        for change in &self.pending_changes {
            let ok = match change {
                PlannedChange::AddRoad { segments, road_id } => {
                    graph.add_road(segments.clone()) == Some(*road_id)
                }
                PlannedChange::RemoveRoad { road_id } => graph.remove_road(*road_id),
                PlannedChange::AssignZone {
                    road_id,
                    segment_id,
                    zone,
                } => graph.assign_zone(*road_id, *segment_id, *zone),
            };
            if !ok {
                return Err("remaining planned changes could not be replayed".into());
            }
        }
        self.overlay_graph = graph;
        Ok(())
    }
}

impl Default for PlanningOverlay {
    fn default() -> Self {
        Self::new(LaneGraph::new())
    }
}

#[cfg(test)]
mod tests {
    use super::PlanningOverlay;
    use crate::{LaneGraph, ZoneType};

    #[test]
    fn staged_edits_only_change_overlay_until_approval() {
        let mut live = LaneGraph::new();
        let road = live.add_road(vec![(1, 2, 10.0)]).unwrap();
        let segment = live.get_segments(road).unwrap()[0].id;
        let mut plan = PlanningOverlay::new(live);
        plan.enable();

        assert!(plan.propose_remove_road(road));
        let extra = plan.propose_add_road(vec![(2, 3, 8.0)]).unwrap();
        assert!(!plan.get_live_graph().road_lanes.contains_key(&extra));
        assert!(plan.get_live_graph().road_lanes.contains_key(&road));
        assert!(plan.get_overlay_graph().road_lanes.contains_key(&extra));
        assert!(!plan.get_overlay_graph().road_lanes.contains_key(&road));
        assert_eq!(plan.pending_change_count(), 2);
        plan.approve_all_changes().unwrap();
        assert!(!plan.get_live_graph().road_lanes.contains_key(&road));
        assert!(plan.get_live_graph().road_lanes.contains_key(&extra));
        assert_eq!(plan.pending_change_count(), 0);

        // Zone edits follow the same stage-then-commit path.
        let mut zone_plan = PlanningOverlay::new(plan.get_live_graph().clone());
        zone_plan.enable();
        let current_segment = zone_plan.get_overlay_graph().get_segments(extra).unwrap()[0].id;
        assert!(zone_plan.propose_zone_assignment(extra, current_segment, ZoneType::Residential));
        assert_eq!(
            zone_plan.get_live_graph().get_zone(extra, current_segment),
            None
        );
        zone_plan.approve_all_changes().unwrap();
        assert_eq!(
            zone_plan.get_live_graph().get_zone(extra, current_segment),
            Some(ZoneType::Residential)
        );
        assert_ne!(segment, current_segment);
    }

    #[test]
    fn inactive_plan_rejects_edits_and_discard_restores_base() {
        let mut plan = PlanningOverlay::default();
        assert_eq!(plan.propose_add_road(vec![(1, 2, 1.0)]), None);
        plan.enable();
        assert!(plan.propose_add_road(vec![(1, 2, 1.0)]).is_some());
        plan.discard_all_changes();
        assert!(!plan.has_pending_changes());
        assert!(plan.get_overlay_graph().road_lanes.is_empty());
    }
}
