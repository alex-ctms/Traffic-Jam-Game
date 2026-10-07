//! Shared simulation types and timing primitives.

use std::time::Duration;

pub type EntityId = u64;

pub struct SimTime {
    pub tick: u64,
}

/// The number of simulation steps run during one second of simulation time.
pub const TICKS_PER_SECOND: u64 = 10;

/// A stable, monotonically increasing simulation step number.
#[derive(Clone, Copy, Debug, Default, Eq, Ord, PartialEq, PartialOrd)]
pub struct Tick(pub u64);

impl Tick {
    pub const ZERO: Self = Self(0);

    pub const fn as_seconds(self) -> f64 {
        self.0 as f64 / TICKS_PER_SECOND as f64
    }

    pub fn from_seconds(seconds: f64) -> Option<Self> {
        if !seconds.is_finite() || seconds < 0.0 {
            return None;
        }
        let ticks = (seconds * TICKS_PER_SECOND as f64).round();
        (ticks <= u64::MAX as f64).then_some(Self(ticks as u64))
    }
}

/// Accumulates wall-clock frame time and runs simulation steps at a fixed rate.
/// Rendering cadence and simulation cadence are independent: each call to
/// [`advance`] runs zero or more complete ticks and retains the fractional time.
#[derive(Clone, Debug)]
pub struct FixedTickRunner {
    tick: Tick,
    accumulated: Duration,
}

impl Default for FixedTickRunner {
    fn default() -> Self {
        Self::new(Tick::ZERO)
    }
}

impl FixedTickRunner {
    pub const TICK_DURATION: Duration = Duration::from_millis(100);

    pub const fn new(starting_tick: Tick) -> Self {
        Self {
            tick: starting_tick,
            accumulated: Duration::ZERO,
        }
    }

    pub const fn tick(&self) -> Tick {
        self.tick
    }

    /// Add elapsed frame time and invoke `step` once per complete simulation tick.
    /// Returns the number of steps run. Remainder time is retained for the next call.
    pub fn advance(&mut self, elapsed: Duration, mut step: impl FnMut(Tick)) -> u64 {
        self.accumulated = self.accumulated.saturating_add(elapsed);
        let mut steps = 0;
        while self.accumulated >= Self::TICK_DURATION {
            self.accumulated -= Self::TICK_DURATION;
            self.tick.0 = self.tick.0.saturating_add(1);
            step(self.tick);
            steps += 1;
        }
        steps
    }
}

pub struct Config {
    pub simulation_speed: f32,
}

pub fn default_config() -> Config {
    Config {
        simulation_speed: 1.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fixed_ticks_are_independent_of_frame_partitioning() {
        let mut steady = FixedTickRunner::default();
        let mut irregular = FixedTickRunner::default();
        let mut steady_ticks = Vec::new();
        let mut irregular_ticks = Vec::new();

        for _ in 0..100 {
            steady.advance(Duration::from_millis(16), |tick| steady_ticks.push(tick));
        }
        for frame in [7, 81, 33, 240, 19, 45, 275, 900] {
            irregular.advance(Duration::from_millis(frame), |tick| {
                irregular_ticks.push(tick)
            });
        }

        assert_eq!(steady_ticks, irregular_ticks);
        assert_eq!(steady.tick(), Tick(16));
        assert_eq!(irregular.tick(), Tick(16));
    }

    #[test]
    fn partial_frame_time_is_carried_until_a_full_tick_is_due() {
        let mut runner = FixedTickRunner::default();
        assert_eq!(runner.advance(Duration::from_millis(99), |_| {}), 0);
        assert_eq!(runner.tick(), Tick::ZERO);
        assert_eq!(runner.advance(Duration::from_millis(1), |_| {}), 1);
        assert_eq!(runner.tick(), Tick(1));
    }

    #[test]
    fn tick_converts_to_and_from_seconds() {
        assert_eq!(Tick(25).as_seconds(), 2.5);
        assert_eq!(Tick::from_seconds(2.5), Some(Tick(25)));
        assert_eq!(Tick::from_seconds(-1.0), None);
        assert_eq!(Tick::from_seconds(f64::NAN), None);
    }
}
