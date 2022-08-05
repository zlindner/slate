use std::sync::Arc;

use crate::{packet_handler::LoginServerPacketHandler, packets, state::State};
use async_trait::async_trait;
use oxide_core::{
    net::{Connection, Events, Packet},
    Db,
};

pub struct LoginServerEventHandler {
    db: Db,
    state: Arc<State>,
}

impl LoginServerEventHandler {
    pub fn new(db: Db, state: Arc<State>) -> Self {
        Self { db, state }
    }
}

#[async_trait]
impl Events for LoginServerEventHandler {
    async fn on_start(&self, addr: &str) {
        log::info!("Login server started @ {}", addr);
    }

    async fn on_connect(&self, connection: &mut Connection) {
        log::info!(
            "Client connected to login server (session {})",
            connection.session_id
        );

        let codec = connection.codec();
        let handshake = packets::handshake(&codec.send, &codec.recv);

        if let Err(e) = connection.write_raw_packet(handshake).await {
            log::error!("Error writing handshake packet: {}", e);
        }
    }

    async fn on_packet(&self, connection: &mut Connection, packet: Packet) {
        log::debug!("Received packet: {}", packet);

        if let Err(e) = LoginServerPacketHandler::get(packet)
            .handle(connection, &self.db.clone(), self.state.clone())
            .await
        {
            log::error!("Handle packet error: {}", e);
        }
    }

    async fn on_disconnect(&self, connection: &mut Connection) {
        log::info!(
            "Client disconnected from login server (session {})",
            connection.session_id
        );

        let state = self.state.clone();
        state.sessions.remove(&connection.session_id);
    }
}
