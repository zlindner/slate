use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub worlds: Vec<WorldConfig>,
}

#[derive(Debug, Deserialize)]
pub struct WorldConfig {
    pub id: i32,
    pub name: String,
    pub channels: i32,
    pub flag: i32, // 0 => nothing, 1 => event, 2 => new, 3 => hot
    pub event_message: String,
    exp_rate: i32,
    drop_rate: i32,
    boss_drop_rate: i32,
    meso_rate: i32,
    quest_rate: i32,
    travel_rate: i32,
    fishing_rate: i32,
}

pub struct World {
    pub config: WorldConfig,
}

impl World {
    pub fn from_config(config: WorldConfig) -> Self {
        World { config }
    }
}
