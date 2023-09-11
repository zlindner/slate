use serde::Deserialize;
use std::fs;

#[derive(Debug, Deserialize)]
pub struct World {
    pub id: i32,
    pub name: String,
    pub channels: i32,
    pub flag: i32,
    pub event_message: String,
    pub recommended_message: String,
    pub exp_rate: i32,
    pub meso_rate: i32,
    pub drop_rate: i32,
    pub boss_drop_rate: i32,
    pub quest_rate: i32,
    pub fishing_rate: i32,
    pub travel_rate: i32,
    pub max_players: i32,
}

#[derive(Debug, Deserialize)]
pub struct Config {
    pub enable_pin: bool,
    pub enable_pic: bool,
    pub worlds: Vec<World>,
}

impl Config {
    pub fn load() -> Self {
        let toml_string =
            fs::read_to_string("config/config.toml").expect("Config should be read from file");
        toml::from_str(&toml_string).expect("Config should be deserialized")
    }
}
