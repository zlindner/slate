use lazy_static::lazy_static;
use serde::Deserialize;

lazy_static! {
    pub static ref CONFIG: OxideConfig = load_oxide_config();
}

#[derive(Debug, Deserialize)]
pub struct OxideConfig {
    pub enable_pin: bool,
    pub enable_pic: bool,
}

fn load_oxide_config() -> OxideConfig {
    let toml = std::fs::read_to_string("config/oxide.toml").unwrap();
    toml::from_str(&toml).unwrap()
}
