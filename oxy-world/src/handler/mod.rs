use crate::client::WorldClient;
use anyhow::Result;
use oxy_core::net::Packet;

mod connect;

pub struct WorldPacketHandler;

impl WorldPacketHandler {
    pub fn new() -> Self {
        Self
    }

    pub async fn handle(&self, mut packet: Packet, client: &mut WorldClient) -> Result<()> {
        let op = packet.read_short();

        match op {
            0x14 => connect::handle(packet, client).await?,
            _ => log::debug!("Unhandled packet: {:02X?}", op),
        }

        Ok(())
    }
}
