use serde::Deserialize;

pub struct Shared {
    pub config: Config,
}

impl Shared {
    pub fn new() -> Self {
        Self {
            config: Self::load_config(),
        }
    }

    fn load_config() -> Config {
        let data = match std::fs::read_to_string("config/config.json") {
            Ok(data) => data,
            Err(e) => {
                panic!("Error loading config: {}", e);
            }
        };

        match serde_json::from_str(&data) {
            Ok(config) => config,
            Err(e) => {
                panic!("Error deserializing config: {}", e);
            }
        }
    }
}

#[derive(Deserialize)]
pub struct Config {
    pub enable_pin: bool,
    pub enable_pic: bool,
    pub worlds: Vec<WorldConfig>,
}

#[derive(Deserialize)]
pub struct WorldConfig {
    pub id: u8,
    pub name: String,
    pub channels: u8,
    pub flag: u8,
    pub event_message: String,
    pub recommended_message: String,
    pub exp_rate: u8,
    pub meso_rate: u8,
    pub drop_rate: u8,
    pub boss_drop_rate: u8,
    pub quest_rate: u8,
    pub fishing_rate: u8,
    pub travel_rate: u8,
    pub max_players: i32,
}
