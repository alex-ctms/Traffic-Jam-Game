//! Analytics module for traffic heatmaps and reporting.

use std::collections::HashMap;
use tj_network::LaneGraph;
use tj_traffic::{TrafficSimulation, Vehicle};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Point {
    pub x: f32,
    pub y: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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
            max_y 
        }
    }

    /// Checks if a point is within this bounding box
    pub fn contains_point(&self, point: &Point) -> bool {
        point.x >= self.min_x && point.x <= self.max_x && 
        point.y >= self.min_y && point.y <= self.max_y
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

impl HeatmapData {
    /// Creates a new heatmap with specified dimensions 
    pub fn new(cell_size: f32, width: usize, height: usize) -> Self {
        let data = vec![0; width * height];
        Self { 
            cell_size, 
            width, 
            height, 
            data 
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
    
    for vehicle in &simulation.vehicles {
        if let Some(lane) = lane_graph.get_lane_by_id(vehicle.lane_id) {
            // Get vehicle position within its lane (0 to lane length)
            let lane_start_x = lane.start_id as f32;
            let lane_start_y = lane.start_id as f32;  // Simplified for now
            let lane_end_x = lane.end_id as f32;
            let lane_end_y = lane.end_id as f32;  // Simplified for now
            
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
    bounding_box: BoundingBox
) -> Vec<&Vehicle> {
    let mut vehicles_in_box = Vec::new();
    let lane_graph = &simulation.network;
    
    for vehicle in &simulation.vehicles {
        if let Some(lane) = lane_graph.get_lane_by_id(vehicle.lane_id) {
            // Calculate position in world coordinates
            let pos_x = lane.start_id as f32;  // Simplified
            let pos_y = lane.start_id as f32;  // Simplified
            
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
}