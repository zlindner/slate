use crate::{
    packet_handler::LoginServerPacketHandler,
    packets, queries,
    state::{Session, State},
};
use async_trait::async_trait;
use oxide_core::{
    net::{Connection, Events, Packet},
    Db,
};
use std::sync::Arc;

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

        let state = self.state.clone();

        // create a new session for the current connection
        if !state.sessions.contains_key(&connection.session_id) {
            state
                .sessions
                .insert(connection.session_id, Session::new(connection.session_id));
        }

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

        if !state.sessions.contains_key(&connection.session_id) {
            return;
        }

        let session = state.sessions.get(&connection.session_id).unwrap();

        if let Err(e) = queries::update_login_state(session.account_id, 0, &self.db).await {
            log::error!("On disconnect error: {}", e);
        }

        state.sessions.remove(&connection.session_id);
    }
}
