use crate::packet_handler::WorldServerPacketHandler;
use async_trait::async_trait;
use oxide_core::{
    net::{Connection, Events, Packet},
    Db, Redis,
};

pub struct WorldServerEventHandler {
    db: Db,
    redis: Redis,
}

impl WorldServerEventHandler {
    pub fn new(db: Db, redis: Redis) -> Self {
        Self { db, redis }
    }
}

#[async_trait]
impl Events for WorldServerEventHandler {
    async fn on_start(&self, addr: &str) {
        log::info!("World server started @ {}", addr);
    }

    async fn on_shutdown(&self) {
        log::info!("World server shutting down...");
    }

    async fn on_connect(&self, connection: &mut Connection) {
        log::info!(
            "Client connected to world server (session {})",
            connection.session_id
        );
    }

    async fn on_packet(&self, connection: &mut Connection, packet: Packet) {
        log::debug!("Received packet: {}", packet);

        if let Err(e) = WorldServerPacketHandler::get(packet)
            .handle(connection, self.db.clone(), self.redis.clone())
            .await
        {
            log::error!("Handle packet error: {}", e);
        }
    }

    async fn on_disconnect(&self, connection: &mut Connection) {
        log::info!(
            "Client disconnected from world server (session {})",
            connection.session_id
        );
    }
}
