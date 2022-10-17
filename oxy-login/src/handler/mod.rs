use anyhow::Result;
use async_trait::async_trait;
use oxy_core::{
    net::{Events, Packet},
    prisma::PrismaClient,
};
use std::{net::SocketAddr, sync::Arc};

mod login;

pub struct EventHandler;

#[async_trait]
impl Events for EventHandler {
    async fn on_start(&self, addr: &str) {
        log::info!("Server started @ {}", addr);
    }

    async fn on_connect(&self, addr: SocketAddr) {
        log::info!("Client connected to server from {}", addr.ip());
    }

    async fn on_packet(&self, packet: Packet, db: &Arc<PrismaClient>) {
        log::debug!("Received: {}", packet);

        if let Err(e) = PacketHandler::handle(packet, db).await {
            log::error!("Error handling packet: {}", e);
        }
    }

    async fn on_disconnect(&self) {
        log::info!("Client disconnected from server");
    }
}

struct PacketHandler;

impl PacketHandler {
    pub async fn handle(mut packet: Packet, db: &Arc<PrismaClient>) -> Result<()> {
        let op = packet.read_short();

        match op {
            0x01 => login::handle(packet, db).await?,
            _ => log::debug!("Unhandled packet: {:02X?}", op),
        }

        Ok(())
    }
}
