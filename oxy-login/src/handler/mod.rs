use anyhow::Result;
use async_trait::async_trait;
use oxy_core::net::{Client, HandlePacket, Packet};
use serde::Deserialize;

mod character_list;
mod create_character;
mod delete_character;
mod login;
mod pin_operation;
mod register_pin;
mod tos;
mod validate_character_name;
mod world_list;
mod world_status;

pub struct PacketHandler {
    pub config: Config,
}

impl PacketHandler {
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

#[async_trait]
impl HandlePacket for PacketHandler {
    async fn handle(&self, mut packet: Packet, client: &mut Client) -> Result<()> {
        log::debug!("Received: {}", packet);
        let op = packet.read_short();

        match op {
            0x01 => login::handle(packet, client, &self.config).await?,
            0x05 => character_list::handle(packet, client, &self.config).await?,
            0x06 => world_status::handle(packet, client, &self.config).await?,
            0x07 => tos::handle(packet, client, &self.config).await?,
            0x09 => pin_operation::handle(packet, client).await?,
            0x0A => register_pin::handle(packet, client).await?,
            0x0B | 0x04 => world_list::handle(packet, client, &self.config).await?,
            0x15 => validate_character_name::handle(packet, client).await?,
            0x16 => create_character::handle(packet, client).await?,
            0x17 => delete_character::handle(packet, client).await?,
            _ => log::debug!("Unhandled packet: {:02X?}", op),
        }

        Ok(())
    }
}

#[derive(Deserialize)]
pub struct Config {
    enable_pin: u8,
    enable_pic: u8,
    worlds: Vec<WorldConfig>,
}

#[derive(Deserialize)]
pub struct WorldConfig {
    id: u8,
    name: String,
    channels: u8,
    flag: u8,
    event_message: String,
    recommended_message: String,
    exp_rate: u8,
    meso_rate: u8,
    drop_rate: u8,
    boss_drop_rate: u8,
    quest_rate: u8,
    fishing_rate: u8,
    travel_rate: u8,
    max_players: i32,
}
