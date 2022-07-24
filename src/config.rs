use once_cell::sync::OnceCell;
use serde::Deserialize;

pub static CONFIG: OnceCell<OxideConfig> = OnceCell::new();

#[derive(Debug, Deserialize)]
pub struct OxideConfig {
    pub enable_pin: u8,
    pub enable_pic: u8,
}

fn load_oxide_config() -> OxideConfig {
    let toml = std::fs::read_to_string("config/oxide.toml").unwrap();
    toml::from_str(&toml).unwrap()
}
