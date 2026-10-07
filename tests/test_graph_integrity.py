import pytest
import sys

# Import the graph module
from tj_network import LaneGraph

# Test class for graph integrity
class TestGraphIntegrity:
    """
    Test cases for graph integrity, ensuring no duplicate edges and that the graph remains connected after each add/remove operation.
    """
    
    def setup_method(self):
        """
        Setup a clean graph before each test.
        """
        self.graph = LaneGraph()
    
    def test_add_edge_no_duplicate(self):
        """
        Test that adding an edge does not create a duplicate edge.
        """
        # Add a segment
        road_id = self.graph.add_segment(1, 2, 10.0)
        assert road_id is not None, "Failed to add segment"
        
        # Check that the graph has the correct number of lanes
        lane_count = self.graph.num_lanes_total()
        assert lane_count == 2, f"Expected 2 lanes for a single segment, got {lane_count}"
        
        # Add the same segment again (should not be allowed)
        with pytest.raises(ValueError, match="road_id already exists"): 
            self.graph.add_segment(1, 2, 10.0)
    
    def test_remove_edge_preserves_connectivity(self):
        """
        Test that removing an edge preserves connectivity.
        """
        # Add a segment
        road_id = self.graph.add_segment(1, 2, 10.0)
        assert road_id is not None, "Failed to add segment"
        
        # Check that the graph has the correct number of lanes
        lane_count = self.graph.num_lanes_total()
        assert lane_count == 2, f"Expected 2 lanes for a single segment, got {lane_count}"
        
        # Remove the segment
        success = self.graph.remove_segment(road_id)
        assert success, "Failed to remove segment"
        
        # Check that the graph has no lanes
        lane_count = self.graph.num_lanes_total()
        assert lane_count == 0, f"Expected 0 lanes after removal, got {lane_count}"
    
    def test_connectivity_after_add_remove(self):
        """
        Test that the graph remains connected after adding and removing segments.
        """
        # Add a segment
        road_id1 = self.graph.add_segment(1, 2, 10.0)
        assert road_id1 is not None, "Failed to add first segment"
        
        # Add another segment
        road_id2 = self.graph.add_segment(2, 3, 15.0)
        assert road_id2 is not None, "Failed to add second segment"
        
        # Check connectivity
        assert self.graph.check_invariants(), "Graph invariants failed after adding segments"
        
        # Remove the first segment
        success = self.graph.remove_segment(road_id1)
        assert success, "Failed to remove first segment"
        
        # Check connectivity after removal
        assert self.graph.check_invariants(), "Graph invariants failed after removing first segment"
    
    def test_invalid_route_connection(self):
        """
        Test that invalid routes are rejected.
        """
        # Add a segment
        road_id = self.graph.add_segment(1, 2, 10.0)
        assert road_id is not None, "Failed to add segment"
        
        # Try to spawn a vehicle on an invalid route
        with pytest.raises(ValueError, match="routes must be connected"): 
            self.graph.spawn_vehicle_on_route([1, 3], 10.0)
    
    def test_duplicate_lane_with_same_start_end(self):
        """
        Test that duplicate lanes with the same start and end are rejected.
        """
        # Add a segment
        road_id = self.graph.add_segment(1, 2, 10.0)
        assert road_id is not None, "Failed to add segment"
        
        # Try to add the same segment again
        with pytest.raises(ValueError, match="road_id already exists"): 
            self.graph.add_segment(1, 2, 10.0)
    
    def test_add_remove_with_multiple_segments(self):
        """
        Test adding and removing multiple segments.
        """
        # Add multiple segments
        road_ids = []
        segments = [(1, 2, 10.0), (2, 3, 15.0), (3, 4, 20.0)]
        for start, end, length in segments:
            road_id = self.graph.add_segment(start, end, length)
            assert road_id is not None, f"Failed to add segment from {start} to {end}"
            road_ids.append(road_id)
        
        # Check total number of lanes
        lane_count = self.graph.num_lanes_total()
        assert lane_count == 6, f"Expected 6 lanes for 3 segments, got {lane_count}"
        
        # Remove the middle segment
        success = self.graph.remove_segment(road_ids[1])
        assert success, "Failed to remove middle segment"
        
        # Check connectivity after removal
        assert self.graph.check_invariants(), "Graph invariants failed after removing middle segment"
        
        # Check that the number of lanes is now 4
        lane_count = self.graph.num_lanes_total()
        assert lane_count == 4, f"Expected 4 lanes after removing middle segment, got {lane_count}"

# Run the tests
if __name__ == "__main__":
    # Create a test instance
    test_suite = TestGraphIntegrity()
    
    # Run the tests
    test_suite.setup_method()
    test_suite.test_add_edge_no_duplicate()
    test_suite.test_remove_edge_preserves_connectivity()
    test_suite.test_connectivity_after_add_remove()
    test_suite.test_invalid_route_connection()
    test_suite.test_duplicate_lane_with_same_start_end()
    test_suite.test_add_remove_with_multiple_segments()
    
    print("All tests passed!")