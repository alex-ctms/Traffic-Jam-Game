# CHANGELOG

## Unreleased

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
