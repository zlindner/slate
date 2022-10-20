use anyhow::Result;
use oxy_core::net::Packet;

use crate::client::CygnusClient;

mod login;

pub struct PacketHandler;

impl PacketHandler {
    pub async fn handle(mut packet: Packet, client: &mut CygnusClient) -> Result<()> {
        log::debug!("Received: {}", packet);
        let op = packet.read_short();

        match op {
            0x00 => login::handle(packet, client).await?,
            _ => log::debug!("Unhandled packet: {:02X?}", op),
        }

        Ok(())
    }
}
