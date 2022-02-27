use serde::Deserialize;

#[derive(Debug)]
pub struct World {
    pub config: WorldConfig,
    pub channels: Vec<Channel>,
}

#[derive(Debug, Deserialize)]
pub struct Config {
    pub worlds: Vec<WorldConfig>,
}

#[derive(Debug, Deserialize)]
pub struct WorldConfig {
    pub id: i32,
    pub name: String,
    pub channels: i32,
    // 0 => nothing, 1 => event, 2 => new, 3 => hot
    pub flag: i32,
    pub event_message: String,
    pub recommended: String,
    exp_rate: i32,
    drop_rate: i32,
    boss_drop_rate: i32,
    meso_rate: i32,
    quest_rate: i32,
    travel_rate: i32,
    fishing_rate: i32,
}

#[derive(Debug)]
pub struct Channel {
    pub id: i32,
    world_id: i32,
}

pub fn load_worlds() -> Vec<World> {
    let mut worlds = Vec::new();
    let toml = std::fs::read_to_string("config/worlds.toml").unwrap();
    let config: Config = toml::from_str(&toml).unwrap();

    for w in config.worlds.into_iter() {
        let world = World::from_config(w);
        worlds.push(world);
    }

    worlds
}

impl World {
    pub fn from_config(config: WorldConfig) -> Self {
        let channels = Self::load_channels(&config);

        World { config, channels }
    }

    pub fn load_channels(config: &WorldConfig) -> Vec<Channel> {
        let mut channels = Vec::new();

        for i in 0..config.channels {
            let channel = Channel::new(i, config.id);
            channels.push(channel);
        }

        channels
    }
}

impl Channel {
    pub fn new(id: i32, world_id: i32) -> Self {
        Channel { id, world_id }
    }
}
