use serde::Deserialize;
use std::fs;

#[derive(Debug, Deserialize)]
pub struct World {
    id: u32,
    name: String,
    channels: u32,
    flag: u32,
    event_message: String,
    recommended_message: String,
    exp_rate: u32,
    meso_rate: u32,
    drop_rate: u32,
    boss_drop_rate: u32,
    quest_rate: u32,
    fishing_rate: u32,
    travel_rate: u32,
    max_players: u32,
}

#[derive(Debug, Deserialize)]
pub struct Config {
    pub enable_pin: bool,
    pub enable_pic: bool,
    worlds: Vec<World>,
}

impl Config {
    pub fn load() -> Self {
        let toml_string =
            fs::read_to_string("config/config.toml").expect("Config should be read from file");
        toml::from_str(&toml_string).expect("Config should be deserialized")
    }
}
