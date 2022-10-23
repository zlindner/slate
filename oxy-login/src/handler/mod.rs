use anyhow::Result;
use async_trait::async_trait;
use oxy_core::net::{Client, HandlePacket, Packet};
use serde::Deserialize;

mod character_list;
mod login;
mod tos;
mod world_list;
mod world_status;

pub struct PacketHandler {
    pub config: HandlerConfig,
}

impl PacketHandler {
    pub fn new() -> Self {
        Self {
            config: Self::load_config(),
        }
    }

    fn load_config() -> HandlerConfig {
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
            0x01 => login::handle(packet, client).await?,
            0x05 => character_list::handle(packet, client, &self.config).await?,
            0x06 => world_status::handle(packet, client, &self.config).await?,
            0x07 => tos::handle(packet, client).await?,
            0x0B => world_list::handle(packet, client, &self.config).await?,
            _ => log::debug!("Unhandled packet: {:02X?}", op),
        }

        Ok(())
    }
}

#[derive(Deserialize)]
pub struct HandlerConfig {
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
