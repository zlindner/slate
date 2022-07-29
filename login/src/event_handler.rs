use crate::{packet_handler::LoginServerPacketHandler, packets};
use async_trait::async_trait;
use oxide_core::{
    net::{Connection, Events, Packet},
    Db,
};

pub struct LoginServerEventHandler {
    db: Db,
}

impl LoginServerEventHandler {
    pub fn new(db: Db) -> Self {
        Self { db }
    }
}

#[async_trait]
impl Events for LoginServerEventHandler {
    async fn on_start(&self, addr: &str) {
        log::info!("Login server started @ {}", addr);
    }

    async fn on_connect(&self, connection: &mut Connection) {
        log::info!("Client connected to login server");

        let codec = connection.codec();
        let handshake = packets::handshake(&codec.send, &codec.recv);

        if let Err(e) = connection.write_raw_packet(handshake).await {
            log::error!("Error writing handshake packet: {}", e);
        }
    }

    async fn on_packet(&self, connection: &mut Connection, packet: Packet) {
        log::debug!("Received packet: {}", packet);

        if let Err(e) = LoginServerPacketHandler::get(packet)
            .handle(connection, &self.db.clone())
            .await
        {
            log::error!("Handle packet error: {}", e);
        }
    }

    async fn on_disconnect(&self, connection: &mut Connection) {
        log::info!("Client disconnected from login server");
    }
}
