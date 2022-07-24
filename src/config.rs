use once_cell::sync::Lazy;
use serde::Deserialize;

pub static CONFIG: Lazy<OxideConfig> = Lazy::new(|| {
    let toml = std::fs::read_to_string("config/oxide.toml").unwrap();
    toml::from_str(&toml).unwrap()
});

#[derive(Debug, Deserialize)]
pub struct OxideConfig {
    pub enable_pin: u8,
    pub enable_pic: u8,
}
