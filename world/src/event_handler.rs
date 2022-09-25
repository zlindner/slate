use crate::{client::Client, packet_handler::WorldServerPacketHandler};
use oxide_core::{net::Packet, Db};

pub struct EventHandler {
    db: Db,
}

impl EventHandler {
    pub fn new(db: Db) -> Self {
        Self { db }
    }

    pub async fn on_start(&self, addr: &str) {
        log::info!("World server started @ {}", addr);
    }

    pub async fn on_shutdown(&self) {
        log::info!("World server shutting down...");
    }

    pub async fn on_connect(&self, client: &Client) {
        log::info!(
            "Client connected to world server (session {})",
            client.session_id
        );
    }

    pub async fn on_packet(&self, client: &mut Client, packet: Packet) {
        log::debug!("Received packet: {}", packet);

        if let Err(e) = WorldServerPacketHandler::get(packet)
            .handle(client, self.db.clone())
            .await
        {
            log::error!("Handle packet error: {}", e);
        }
    }

    pub async fn on_disconnect(&self, client: &mut Client) {
        log::info!(
            "Client disconnected from world server (session {})",
            client.session_id
        );
    }
}
