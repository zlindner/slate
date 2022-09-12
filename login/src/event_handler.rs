use crate::{client::Client, packet_handler::LoginServerPacketHandler, queries};
use oxide_core::{net::Packet, Db};

pub struct EventHandler {
    db: Db,
}

impl EventHandler {
    pub fn new(db: Db) -> Self {
        Self { db }
    }

    pub async fn on_start(&self, addr: &str) {
        log::info!("Login server started @ {}", addr);
    }

    pub async fn on_shutdown(&self) {
        log::info!("Login server shutting down...");

        if let Err(e) = queries::logout_all(&self.db).await {
            log::error!("Error executing logout_all: {}", e);
        }

        // TODO delete all sessions
    }

    pub async fn on_connect(&self, client: &mut Client) {
        log::info!(
            "Client connected to login server (session {})",
            client.session.id
        );
    }

    pub async fn on_packet(&self, client: &mut Client, packet: Packet) {
        log::debug!("Received packet: {}", packet);

        if let Err(e) = LoginServerPacketHandler::get(packet)
            .handle(client, self.db.clone())
            .await
        {
            log::error!("Handle packet error: {}", e);
        }
    }

    pub async fn on_disconnect(&self, client: &mut Client) {
        log::info!(
            "Client disconnected from login server (session {})",
            client.session.id
        );

        if let Err(e) = queries::update_login_state(client.session.account_id, 0, &self.db).await {
            log::error!("Error updating login state: {}", e);
        }
    }
}
