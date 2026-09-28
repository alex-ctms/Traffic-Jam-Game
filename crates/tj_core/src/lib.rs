// Core types, IDs, time, config

pub type EntityId = u64;

pub struct SimTime {
    pub tick: u64,
}

pub struct Config {
    pub simulation_speed: f32,
}

pub fn default_config() -> Config {
    Config {
        simulation_speed: 1.0,
    }
}
