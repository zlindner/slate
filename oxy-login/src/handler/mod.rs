use anyhow::Result;
use async_trait::async_trait;
use oxy_core::{
    net::{Client, Events, Packet},
    prisma::PrismaClient,
};
use std::sync::Arc;

mod login;

pub struct EventHandler;

#[async_trait]
impl Events for EventHandler {
    async fn on_start(&self, addr: &str) {
        log::info!("Server started @ {}", addr);
    }

    async fn on_connect(&self, client: &Client) {
        log::info!("Client connected to server (session {})", client.session_id);
    }

    async fn on_packet(&self, packet: Packet, client: &mut Client, db: &Arc<PrismaClient>) {
        log::debug!("Received: {}", packet);

        if let Err(e) = PacketHandler::handle(packet, client, db).await {
            log::error!("Error handling packet: {}", e);
        }
    }

    async fn on_disconnect(&self, client: &Client) {
        log::info!(
            "Client disconnected from server (session {})",
            client.session_id
        );
    }
}

struct PacketHandler;

impl PacketHandler {
    pub async fn handle(
        mut packet: Packet,
        client: &mut Client,
        db: &Arc<PrismaClient>,
    ) -> Result<()> {
        let op = packet.read_short();

        match op {
            0x01 => login::handle(packet, db).await?,
            _ => log::debug!("Unhandled packet: {:02X?}", op),
        }

        Ok(())
    }
}
