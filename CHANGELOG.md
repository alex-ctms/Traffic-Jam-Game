# CHANGELOG

## Unreleased

- Added lane segment records and lane-ID adjacency tracking to the network graph, including segment removal.
- Added road-segment zone assignment and lookup for residential, commercial, office, industrial, and farm designations.
- Added an end-to-end traffic signal coordination test covering red-light stops and green-light vehicle flow.
- Added routed vehicle lane progression with signal stop lines and per-tick yield instructions.
- Added a headless Bevy demand-generation integration test that verifies ten ticks queue every generated trip.
