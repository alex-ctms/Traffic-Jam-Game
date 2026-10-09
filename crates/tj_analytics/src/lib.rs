//! Analytics module for traffic heatmaps and reporting.

use std::io;
use std::path::Path;
use tj_traffic::{TrafficSimulation, Vehicle};

/// City traffic and service metrics captured at a monthly reporting boundary.
/// Metrics not yet produced by the simulation can remain zero until wired in.
#[derive(Debug, Clone, Default, PartialEq, serde::Serialize)]
pub struct MonthlyTrafficStatistics {
    pub year: u32,
    pub month: u8,
    pub population_change: i64,
    pub city_score: f64,
    pub average_commute_minutes: f64,
    pub average_delay_minutes: f64,
    pub crashes: u64,
    pub fatalities: u64,
    pub tickets: u64,
    pub happiness: f64,
    pub transit_ridership: u64,
    pub road_length: f64,
    pub construction_projects: u64,
    pub emergency_response_minutes: f64,
    pub average_land_value: f64,
    pub business_activity: u64,
    pub revenue: i64,
    pub engineering_budget: i64,
    pub ordinance_results: Vec<String>,
    pub neighborhood_issues: Vec<String>,
}

/// Writes one monthly report as pretty-printed JSON, replacing an existing file.
pub fn write_monthly_traffic_report(
    path: impl AsRef<Path>,
    statistics: &MonthlyTrafficStatistics,
) -> io::Result<()> {
    let json = serde_json::to_vec_pretty(statistics).map_err(io::Error::other)?;
    std::fs::write(path, json)
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Point {
    pub x: f32,
    pub y: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BoundingBox {
    pub min_x: f32,
    pub max_x: f32,
    pub min_y: f32,
    pub max_y: f32,
}

impl BoundingBox {
    /// Creates a new bounding box
    pub fn new(min_x: f32, max_x: f32, min_y: f32, max_y: f32) -> Self {
        Self {
            min_x,
            max_x,
            min_y,
            max_y,
        }
    }

    /// Checks if a point is within this bounding box
    pub fn contains_point(&self, point: &Point) -> bool {
        point.x >= self.min_x
            && point.x <= self.max_x
            && point.y >= self.min_y
            && point.y <= self.max_y
    }
}

/// Heatmap data structure representing vehicle density in different grid cells
#[derive(Debug, Clone)]
pub struct HeatmapData {
    /// Grid cell size (in meters)
    pub cell_size: f32,
    /// Grid dimensions
    pub width: usize,
    pub height: usize,
    /// Density values for each grid cell
    pub data: Vec<u32>,
}

/// Vehicle density on one physical road segment for a single simulation tick.
/// Vehicles travelling in either direction contribute to the same segment.
#[derive(Debug, Clone, PartialEq)]
pub struct LaneSegmentDensity {
    pub road_id: u32,
    pub segment_id: u64,
    pub start_id: u32,
    pub end_id: u32,
    pub vehicle_count: u32,
    /// Vehicles per kilometer; zero for a zero-length segment.
    pub vehicles_per_kilometer: f32,
}

/// A point-in-time congestion heatmap keyed by physical lane segment.
#[derive(Debug, Clone, PartialEq)]
pub struct LaneSegmentHeatmap {
    pub tick: u64,
    pub segments: Vec<LaneSegmentDensity>,
}

/// Aggregate current vehicle locations into physical road segments.
///
/// Call once after each simulation tick to retain a per-tick heatmap snapshot.
/// Segments without vehicles are included with zero density so callers can render
/// the complete road network consistently.
pub fn generate_lane_segment_heatmap(simulation: &TrafficSimulation) -> LaneSegmentHeatmap {
    let mut segments = Vec::new();
    for (road_id, road_segments) in &simulation.network.road_segments {
        for segment in road_segments {
            let vehicle_count = simulation
                .vehicles()
                .iter()
                .filter(|vehicle| {
                    simulation
                        .network
                        .get_lane_by_id(vehicle.lane_id)
                        .is_some_and(|lane| {
                            lane.road_id == *road_id
                                && ((lane.start_id == segment.start_id
                                    && lane.end_id == segment.end_id)
                                    || (lane.start_id == segment.end_id
                                        && lane.end_id == segment.start_id))
                        })
                })
                .count() as u32;
            let vehicles_per_kilometer = if segment.length > 0.0 {
                vehicle_count as f32 * 1000.0 / segment.length
            } else {
                0.0
            };
            segments.push(LaneSegmentDensity {
                road_id: *road_id,
                segment_id: segment.id,
                start_id: segment.start_id,
                end_id: segment.end_id,
                vehicle_count,
                vehicles_per_kilometer,
            });
        }
    }
    segments.sort_by_key(|segment| (segment.road_id, segment.segment_id));
    LaneSegmentHeatmap {
        tick: simulation.tick_count(),
        segments,
    }
}

impl HeatmapData {
    /// Creates a new heatmap with specified dimensions
    pub fn new(cell_size: f32, width: usize, height: usize) -> Self {
        let data = vec![0; width * height];
        Self {
            cell_size,
            width,
            height,
            data,
        }
    }

    /// Gets the density at a specific grid cell index
    pub fn get_density(&self, x: usize, y: usize) -> u32 {
        if x < self.width && y < self.height {
            self.data[y * self.width + x]
        } else {
            0
        }
    }

    /// Sets the density at a specific grid cell index
    pub fn set_density(&mut self, x: usize, y: usize, density: u32) {
        if x < self.width && y < self.height {
            self.data[y * self.width + x] = density;
        }
    }

    /// Increments the density at a specific grid cell index
    pub fn increment_density(&mut self, x: usize, y: usize) {
        if x < self.width && y < self.height {
            let index = y * self.width + x;
            self.data[index] += 1;
        }
    }

    /// Converts a world coordinate to grid cell coordinates
    pub fn world_to_grid(&self, x: f32, y: f32) -> (usize, usize) {
        let grid_x = (x / self.cell_size).floor() as usize;
        let grid_y = (y / self.cell_size).floor() as usize;
        (grid_x, grid_y)
    }
}

/// Generates heatmap data from simulation state
pub fn generate_heatmap_data(
    simulation: &TrafficSimulation,
    bounding_box: BoundingBox,
    cell_size: f32,
) -> HeatmapData {
    // Calculate grid dimensions based on bounding box and cell size
    let width = ((bounding_box.max_x - bounding_box.min_x) / cell_size).ceil() as usize;
    let height = ((bounding_box.max_y - bounding_box.min_y) / cell_size).ceil() as usize;

    // Create heatmap with specified cell size
    let mut heatmap = HeatmapData::new(cell_size, width, height);

    // Process vehicles in the simulation to generate density data
    let lane_graph = &simulation.network;

    for vehicle in simulation.vehicles() {
        if let Some(lane) = lane_graph.get_lane_by_id(vehicle.lane_id) {
            // Get vehicle position within its lane (0 to lane length)
            let lane_start_x = lane.start_id as f32;
            let lane_start_y = lane.start_id as f32; // Simplified for now
            let lane_end_x = lane.end_id as f32;
            let lane_end_y = lane.end_id as f32; // Simplified for now

            // Calculate vehicle position in world coordinates (simplified)
            // In a real implementation this would need proper spatial mapping
            let pos_x = lane_start_x + (lane_end_x - lane_start_x) * vehicle.position / lane.length;
            let pos_y = lane_start_y + (lane_end_y - lane_start_y) * vehicle.position / lane.length;

            let point = Point { x: pos_x, y: pos_y };

            // Check if the point is within the requested bounding box
            if bounding_box.contains_point(&point) {
                // Convert to grid coordinates and increment density
                let (grid_x, grid_y) = heatmap.world_to_grid(pos_x, pos_y);
                heatmap.increment_density(grid_x, grid_y);
            }
        }
    }

    heatmap
}

/// Gets vehicles within a specific bounding box from a simulation
pub fn get_vehicles_in_bounding_box(
    simulation: &TrafficSimulation,
    bounding_box: BoundingBox,
) -> Vec<&Vehicle> {
    let mut vehicles_in_box = Vec::new();
    let lane_graph = &simulation.network;

    for vehicle in simulation.vehicles() {
        if let Some(lane) = lane_graph.get_lane_by_id(vehicle.lane_id) {
            // Calculate position in world coordinates
            let pos_x = lane.start_id as f32; // Simplified
            let pos_y = lane.start_id as f32; // Simplified

            let point = Point { x: pos_x, y: pos_y };

            if bounding_box.contains_point(&point) {
                vehicles_in_box.push(vehicle);
            }
        }
    }

    vehicles_in_box
}

/// API endpoint function that returns heatmap data for a requested bounding box
/// This function is designed to be called as an API endpoint from the backend service.
pub fn get_heatmap_data_for_bounding_box(
    simulation: &TrafficSimulation,
    min_x: f32,
    max_x: f32,
    min_y: f32,
    max_y: f32,
    cell_size: f32,
) -> HeatmapData {
    let bounding_box = BoundingBox::new(min_x, max_x, min_y, max_y);
    generate_heatmap_data(simulation, bounding_box, cell_size)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tj_network::LaneGraph;

    #[test]
    fn test_heatmap_create() {
        let heatmap = HeatmapData::new(10.0, 5, 5);
        assert_eq!(heatmap.cell_size, 10.0);
        assert_eq!(heatmap.width, 5);
        assert_eq!(heatmap.height, 5);
        assert_eq!(heatmap.data.len(), 25);
    }

    #[test]
    fn test_heatmap_density_operations() {
        let mut heatmap = HeatmapData::new(5.0, 3, 3);

        // Test setting density
        heatmap.set_density(1, 1, 42);
        assert_eq!(heatmap.get_density(1, 1), 42);

        // Test incrementing density
        heatmap.increment_density(1, 1);
        assert_eq!(heatmap.get_density(1, 1), 43);

        // Test boundary conditions
        assert_eq!(heatmap.get_density(5, 5), 0); // Out of bounds should return 0
    }

    #[test]
    fn test_bounding_box_contains_point() {
        let bbox = BoundingBox::new(0.0, 10.0, 0.0, 10.0);

        assert!(bbox.contains_point(&Point { x: 5.0, y: 5.0 }));
        assert!(bbox.contains_point(&Point { x: 0.0, y: 0.0 }));
        assert!(bbox.contains_point(&Point { x: 10.0, y: 10.0 }));
        assert!(!bbox.contains_point(&Point { x: -1.0, y: 5.0 }));
        assert!(!bbox.contains_point(&Point { x: 5.0, y: -1.0 }));
        assert!(!bbox.contains_point(&Point { x: 15.0, y: 5.0 }));
        assert!(!bbox.contains_point(&Point { x: 5.0, y: 15.0 }));
    }

    #[test]
    fn test_generate_heatmap_data() {
        let mut network = LaneGraph::new();
        network.add_segment(1, 2, 10.0).unwrap();

        let mut simulation = TrafficSimulation::new(network);
        // Add a signal to make it more realistic
        simulation.add_signal(2, tj_traffic::TrafficSignal::new(1, 1));

        let bbox = BoundingBox::new(0.0, 5.0, 0.0, 5.0);
        let heatmap = generate_heatmap_data(&simulation, bbox, 5.0);

        assert_eq!(heatmap.cell_size, 5.0);
        assert_eq!(heatmap.width, 1);
        assert_eq!(heatmap.height, 1);
    }

    #[test]
    fn test_get_heatmap_data_for_bounding_box() {
        let mut network = LaneGraph::new();
        network.add_segment(1, 2, 10.0).unwrap();

        let mut simulation = TrafficSimulation::new(network);
        // Add a signal to make it more realistic
        simulation.add_signal(2, tj_traffic::TrafficSignal::new(1, 1));

        let heatmap = get_heatmap_data_for_bounding_box(&simulation, 0.0, 5.0, 0.0, 5.0, 5.0);
        assert_eq!(heatmap.cell_size, 5.0);
        assert_eq!(heatmap.width, 1);
        assert_eq!(heatmap.height, 1);
    }

    #[test]
    fn lane_segment_heatmap_aggregates_both_directions_for_each_tick() {
        let mut network = LaneGraph::new();
        let road_id = network.add_segment(10, 20, 500.0).unwrap();
        let lanes = network.get_road_lanes(road_id).unwrap();
        let forward_lane = lanes[0].id;
        let backward_lane = lanes[1].id;
        let mut simulation = TrafficSimulation::new(network);
        simulation.spawn_vehicle(forward_lane, 1.0).unwrap();
        simulation.spawn_vehicle(backward_lane, 1.0).unwrap();

        let first_tick = generate_lane_segment_heatmap(&simulation);
        assert_eq!(first_tick.tick, 0);
        assert_eq!(first_tick.segments.len(), 1);
        assert_eq!(first_tick.segments[0].vehicle_count, 2);
        assert_eq!(first_tick.segments[0].vehicles_per_kilometer, 4.0);

        simulation.tick();
        let next_tick = generate_lane_segment_heatmap(&simulation);
        assert_eq!(next_tick.tick, 1);
        assert_eq!(next_tick.segments[0].vehicle_count, 2);
    }

    #[test]
    fn lane_segment_heatmap_density_increases_proportionally_with_vehicle_count() {
        let mut network = LaneGraph::new();
        let road_id = network.add_segment(10, 20, 500.0).unwrap();
        let lane_id = network.get_road_lanes(road_id).unwrap()[0].id;
        let mut simulation = TrafficSimulation::new(network);

        let mut densities = Vec::new();
        let mut spawned = 0;
        for vehicle_count in [1, 2, 4] {
            while spawned < vehicle_count {
                simulation.spawn_vehicle(lane_id, 1.0).unwrap();
                spawned += 1;
            }
            let heatmap = generate_lane_segment_heatmap(&simulation);
            let segment = &heatmap.segments[0];
            assert_eq!(segment.vehicle_count, vehicle_count);
            densities.push(segment.vehicles_per_kilometer);
        }

        assert_eq!(densities, [2.0, 4.0, 8.0]);
        assert_eq!(densities[1] / densities[0], 2.0);
        assert_eq!(densities[2] / densities[0], 4.0);
    }
}

#[cfg(test)]
mod monthly_report_tests {
    use super::{write_monthly_traffic_report, MonthlyTrafficStatistics};

    #[test]
    fn writes_monthly_report_with_stable_metric_layout() {
        let path = std::env::temp_dir().join(format!(
            "tj-monthly-report-{}-{}.json",
            std::process::id(),
            std::thread::current().name().unwrap_or("test")
        ));
        let report = MonthlyTrafficStatistics {
            year: 2026,
            month: 9,
            population_change: 125,
            city_score: 82.5,
            average_commute_minutes: 24.0,
            crashes: 3,
            ordinance_results: vec!["passed: transit priority".into()],
            neighborhood_issues: vec!["north: congestion".into()],
            ..Default::default()
        };

        write_monthly_traffic_report(&path, &report).unwrap();
        let contents = std::fs::read_to_string(&path).unwrap();
        let json: serde_json::Value = serde_json::from_str(&contents).unwrap();
        std::fs::remove_file(path).unwrap();

        assert_eq!(json["year"], 2026);
        assert_eq!(json["month"], 9);
        assert_eq!(json["population_change"], 125);
        assert_eq!(json["city_score"], 82.5);
        assert_eq!(json["average_commute_minutes"], 24.0);
        assert_eq!(json["crashes"], 3);
        assert_eq!(json["ordinance_results"][0], "passed: transit priority");
        assert_eq!(json["neighborhood_issues"][0], "north: congestion");
        assert!(json.get("revenue").is_some());
        assert!(json.get("engineering_budget").is_some());
    }
}
