use anyhow::Result;
use async_trait::async_trait;
use oxy_core::{
    net::{Events, Packet},
    prisma::PrismaClient,
};
use std::sync::Arc;

mod connect;

pub struct EventHandler;

#[async_trait]
impl Events for EventHandler {
    async fn on_start(&self, addr: &str) {
        log::debug!("Server started @ {}", addr);
    }

    async fn on_connect(&self) {
        log::debug!("Client connected to server");
    }

    async fn on_packet(&self, packet: Packet, db: &Arc<PrismaClient>) {
        log::debug!("Received: {}", packet);

        if let Err(e) = PacketHandler::handle(packet, db).await {
            log::error!("Error handling packet: {}", e);
        }
    }

    async fn on_disconnect(&self) {
        log::error!("Client disconnected from server");
    }
}

struct PacketHandler;

impl PacketHandler {
    pub async fn handle(mut packet: Packet, db: &Arc<PrismaClient>) -> Result<()> {
        let op = packet.read_short();

        match op {
            0x14 => connect::handle(packet, db).await?,
            _ => log::debug!("Unhandled packet: {:02X?}", op),
        }

        Ok(())
    }
}
