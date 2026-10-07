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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VehicleInstruction {
    Proceed,
    Yield,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Vehicle {
    pub id: u64,
    pub lane_id: u64,
    pub position: f32,
    pub speed: f32,
    /// Ordered lane IDs the vehicle will follow, including its current lane.
    pub route: Vec<u64>,
    route_index: usize,
}

impl Vehicle {
    /// Advance one simulation tick, stopping at the lane end for red signals or yield commands.
    pub fn step(
        &mut self,
        network: &LaneGraph,
        signal_state: SignalState,
        instruction: VehicleInstruction,
    ) {
        let Some(lane) = network.get_lane_by_id(self.lane_id) else {
            return;
        };
        let distance_to_end = (lane.length - self.position).max(0.0);
        if distance_to_end <= self.speed.max(0.0)
            && (signal_state == SignalState::Red || instruction == VehicleInstruction::Yield)
        {
            self.position = lane.length;
            return;
        }

        self.position = (self.position + self.speed.max(0.0)).min(lane.length);
        if self.position >= lane.length && self.route_index + 1 < self.route.len() {
            let next_lane_id = self.route[self.route_index + 1];
            if network
                .get_lane_by_id(next_lane_id)
                .is_some_and(|next| next.start_id == lane.end_id)
            {
                self.route_index += 1;
                self.lane_id = next_lane_id;
                self.position = 0.0;
            }
        }
    }
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
        self.spawn_vehicle_on_route(&[lane_id], speed)
    }

    /// Spawn a vehicle with an ordered, connected lane route.
    pub fn spawn_vehicle_on_route(&mut self, route: &[u64], speed: f32) -> Option<u64> {
        if route.is_empty()
            || route
                .iter()
                .any(|id| self.network.get_lane_by_id(*id).is_none())
        {
            return None;
        }
        if route.windows(2).any(|pair| {
            self.network
                .get_lane_by_id(pair[0])
                .zip(self.network.get_lane_by_id(pair[1]))
                .is_none_or(|(from, to)| from.end_id != to.start_id)
        }) {
            return None;
        }
        let id = self.next_vehicle_id;
        self.next_vehicle_id += 1;
        self.vehicles.push(Vehicle {
            id,
            lane_id: route[0],
            position: 0.0,
            speed,
            route: route.to_vec(),
            route_index: 0,
        });
        Some(id)
    }

    pub fn vehicle(&self, id: u64) -> Option<&Vehicle> {
        self.vehicles.iter().find(|vehicle| vehicle.id == id)
    }

    pub fn tick(&mut self) {
        self.tick_with_instructions(&HashMap::new());
    }

    /// Advance every vehicle once, applying optional per-vehicle yield instructions.
    pub fn tick_with_instructions(&mut self, instructions: &HashMap<u64, VehicleInstruction>) {
        for vehicle in &mut self.vehicles {
            let Some(lane) = self.network.get_lane_by_id(vehicle.lane_id) else {
                continue;
            };
            let signal_state = self
                .signals
                .get(&lane.end_id)
                .map_or(SignalState::Green, |signal| signal.state_at(self.tick));

            let instruction = instructions
                .get(&vehicle.id)
                .copied()
                .unwrap_or(VehicleInstruction::Proceed);
            vehicle.step(&self.network, signal_state, instruction);
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

    #[test]
    fn vehicle_waits_for_green_then_advances_to_the_next_lane() {
        let mut network = LaneGraph::new();
        let first_road = network.add_segment(1, 2, 10.0).unwrap();
        let second_road = network.add_segment(2, 3, 10.0).unwrap();
        let first_lane = network.get_road_lanes(first_road).unwrap()[0].id;
        let next_lane = network
            .get_road_lanes(second_road)
            .unwrap()
            .iter()
            .find(|lane| lane.start_id == 2)
            .unwrap()
            .id;
        let mut simulation = TrafficSimulation::new(network);
        simulation.add_signal(2, TrafficSignal::new(1, 1));
        let vehicle_id = simulation
            .spawn_vehicle_on_route(&[first_lane, next_lane], 10.0)
            .unwrap();

        simulation.tick(); // red: reaches the stop line but cannot enter the next lane
        let vehicle = simulation.vehicle(vehicle_id).unwrap();
        assert_eq!(vehicle.lane_id, first_lane);
        assert_eq!(vehicle.position, 10.0);

        simulation.tick(); // green
        let vehicle = simulation.vehicle(vehicle_id).unwrap();
        assert_eq!(vehicle.lane_id, next_lane);
        assert_eq!(vehicle.position, 0.0);
    }

    #[test]
    fn yield_instruction_holds_vehicle_at_lane_end() {
        let mut network = LaneGraph::new();
        let first_road = network.add_segment(1, 2, 10.0).unwrap();
        let second_road = network.add_segment(2, 3, 10.0).unwrap();
        let first_lane = network.get_road_lanes(first_road).unwrap()[0].id;
        let next_lane = network
            .get_road_lanes(second_road)
            .unwrap()
            .iter()
            .find(|lane| lane.start_id == 2)
            .unwrap()
            .id;
        let mut simulation = TrafficSimulation::new(network);
        let vehicle_id = simulation
            .spawn_vehicle_on_route(&[first_lane, next_lane], 10.0)
            .unwrap();
        let instructions = HashMap::from([(vehicle_id, VehicleInstruction::Yield)]);

        simulation.tick_with_instructions(&instructions);
        let vehicle = simulation.vehicle(vehicle_id).unwrap();
        assert_eq!(vehicle.lane_id, first_lane);
        assert_eq!(vehicle.position, 10.0);

        simulation.tick();
        assert_eq!(simulation.vehicle(vehicle_id).unwrap().lane_id, next_lane);
    }

    #[test]
    fn vehicle_routes_must_be_connected() {
        let mut network = LaneGraph::new();
        let first = network.add_segment(1, 2, 10.0).unwrap();
        let disconnected = network.add_segment(3, 4, 10.0).unwrap();
        let first_lane = network.get_road_lanes(first).unwrap()[0].id;
        let disconnected_lane = network.get_road_lanes(disconnected).unwrap()[0].id;
        let mut simulation = TrafficSimulation::new(network);

        assert!(simulation
            .spawn_vehicle_on_route(&[first_lane, disconnected_lane], 1.0)
            .is_none());
    }
}
