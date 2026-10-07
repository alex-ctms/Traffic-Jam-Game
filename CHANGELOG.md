# CHANGELOG

## Unreleased

- M1.1 (backend): Add a fixed‑time simulation tick system that steps simulation logic at a constant rate regardless of frame rendering — t_d7177750
- M6.1 (backend): Implement a planning mode that overlays proposed road edits on top of the current network without modifying live data until approval — t_397a67a4. The implementation correctly provides a planning mode that overlays edits on the live network without affecting live data until an explicit approval step, and it includes automated tests that verify t
- M5.4 (frontend): Add a frontend overlay that visualizes the congestion heatmap with color gradients — t_0b814f0b. Implementation adds a CongestionHeatmapOverlay component, plugin, and rendering system that visualizes congestion with a blue‑yellow‑red gradient, and includes a targeted test confirming cell renderin
- M1.3 (testing): Write unit tests that assert there are no duplicate edges and that the graph remains connected after each add/remove operation — t_e939ee0e
- M5.1 (backend): Generate a congestion heatmap by aggregating the density of vehicles per lane segment each tick — t_e947965f. The changes implement per‑tick lane‑segment congestion aggregation and provide a unit test plus verify.sh entry that pass.
- M5.4 (frontend): Add a frontend overlay that visualizes the congestion heatmap with color gradients — t_4a07e82d
- M6.5 (testing): Unit test that after approval the network graph reflects the planned changes and all dependent data (zone calculations, traffic paths) update co — t_261d3b45. The implementation adds a focused unit test that verifies a network change is correctly applied, updates zone data, and propagates to traffic paths. The test matches the acceptance criterion verbatim.
- M2.3 (backend): Create a console command that creates a simple two‑point road and automatically applies a default residential zone — t_0063c01b. The implementation provides a console command `/create_road` that constructs a two‑point residential road via `LaneGraph::add_segment` and zones it residential, and includes a unit test asserting corr
- M5.2 (backend): Provide an API endpoint that returns heatmap data for a requested bounding box — t_4b156db4
- M5.5 (infra): Implement a monthly traffic statistics reporter that writes a JSON file to disk and is unit‑tested for data layout — t_fc07e530. Implementation fully satisfies the acceptance criteria: the monthly traffic statistics reporter writes JSON to disk, includes a unit test verifying the data layout, and the verification gate in ops/ve
- M3.5 (backend): Expose a console command `/pop_stats` that prints the current population and job counts — t_fac7f84b. The implementation correctly adds a `/pop_stats` console command that outputs the current population and job counts, and a unit test validates the output. All acceptance criteria are satisfied.
- M2.1 (backend): Expose an API to assign a zone type (`Residential`, `Commercial`, `Industrial`, etc.) to a road segment — t_9ecf68e3
- M4.2 (backend): Create a `Vehicle` component that steps through lanes each tick, obeying traffic signal phases and yielding instructions — t_e145175a
- M1.1 (backend): Add a fixed‑time simulation tick system that steps simulation logic at a constant rate regardless of frame rendering — t_06f3a89a
- M3.4 (testing): Write an integration test that runs a headless Bevy simulation for 10 ticks and verifies all generated trips are queued and no panics occur — t_0e81eb5d
- M3.3 (testing): Unit test that the total number of active trips equals the number of households multiplied by the average trips per day — t_e61ec6bb
- M1.2 (backend): Implement a `LaneSegment` struct and `LaneGraph` that supports adding and removing road segments while maintaining a reliable adjacency list — t_57046b7f
- M1.1 (backend): Add a fixed‑time simulation tick system that steps simulation logic at a constant rate regardless of frame rendering — t_c5d291a5
- Added lane segment records and lane-ID adjacency tracking to the network graph, including segment removal.
- Added road-segment zone assignment and lookup for residential, commercial, office, industrial, and farm designations.
- Added an end-to-end traffic signal coordination test covering red-light stops and green-light vehicle flow.
- Added routed vehicle lane progression with signal stop lines and per-tick yield instructions.
- Added a headless Bevy demand-generation integration test that verifies ten ticks queue every generated trip.
