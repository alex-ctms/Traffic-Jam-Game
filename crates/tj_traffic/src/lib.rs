//! Deterministic, headless traffic simulation primitives.

use std::collections::HashMap;

use tj_network::LaneGraph;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SignalState {
    Red,
    Green,
}

#[derive(Debug, Clone, Copy)]
pub struct TrafficSignal {
    pub red_ticks: u64,
    pub green_ticks: u64,
}

impl TrafficSignal {
    pub fn new(red_ticks: u64, green_ticks: u64) -> Self {
        assert!(
            red_ticks + green_ticks > 0,
            "signal cycle must not be empty"
        );
        Self {
            red_ticks,
            green_ticks,
        }
    }

    pub fn state_at(&self, tick: u64) -> SignalState {
        if tick % (self.red_ticks + self.green_ticks) < self.red_ticks {
            SignalState::Red
        } else {
            SignalState::Green
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Vehicle {
    pub id: u64,
    pub lane_id: u64,
    pub position: f32,
    pub speed: f32,
}

/// A small simulation loop that advances vehicles toward the end of their lane.
/// A signal at that lane's destination intersection controls whether they move.
#[derive(Debug, Default)]
pub struct TrafficSimulation {
    pub network: LaneGraph,
    signals: HashMap<u32, TrafficSignal>,
    vehicles: Vec<Vehicle>,
    tick: u64,
    next_vehicle_id: u64,
}

impl TrafficSimulation {
    pub fn new(network: LaneGraph) -> Self {
        Self {
            network,
            ..Self::default()
        }
    }

    pub fn add_signal(&mut self, intersection_id: u32, signal: TrafficSignal) {
        self.signals.insert(intersection_id, signal);
    }

    pub fn spawn_vehicle(&mut self, lane_id: u64, speed: f32) -> Option<u64> {
        self.network.get_lane_by_id(lane_id)?;
        let id = self.next_vehicle_id;
        self.next_vehicle_id += 1;
        self.vehicles.push(Vehicle {
            id,
            lane_id,
            position: 0.0,
            speed,
        });
        Some(id)
    }

    pub fn vehicle(&self, id: u64) -> Option<&Vehicle> {
        self.vehicles.iter().find(|vehicle| vehicle.id == id)
    }

    pub fn tick(&mut self) {
        for vehicle in &mut self.vehicles {
            let Some(lane) = self.network.get_lane_by_id(vehicle.lane_id) else {
                continue;
            };
            let signal_state = self
                .signals
                .get(&lane.end_id)
                .map_or(SignalState::Green, |signal| signal.state_at(self.tick));

            if signal_state == SignalState::Green {
                vehicle.position = (vehicle.position + vehicle.speed.max(0.0)).min(lane.length);
            }
        }
        self.tick += 1;
    }

    pub fn tick_count(&self) -> u64 {
        self.tick
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn signal_starts_red_and_then_turns_green() {
        let signal = TrafficSignal::new(2, 3);
        assert_eq!(signal.state_at(0), SignalState::Red);
        assert_eq!(signal.state_at(1), SignalState::Red);
        assert_eq!(signal.state_at(2), SignalState::Green);
        assert_eq!(signal.state_at(4), SignalState::Green);
        assert_eq!(signal.state_at(5), SignalState::Red);
    }
}
