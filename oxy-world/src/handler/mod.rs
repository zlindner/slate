use anyhow::Result;
use async_trait::async_trait;
use oxy_core::net::{Client, HandlePacket, Packet};

mod connect;

pub struct PacketHandler;

#[async_trait]
impl HandlePacket for PacketHandler {
    async fn handle(&self, mut packet: Packet, client: &mut Client) -> Result<()> {
        let op = packet.read_short();

        match op {
            0x14 => connect::handle(packet, client).await?,
            _ => log::debug!("Unhandled packet: {:02X?}", op),
        }

        Ok(())
    }
}
