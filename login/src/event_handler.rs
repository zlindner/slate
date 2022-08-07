use crate::{packet_handler::LoginServerPacketHandler, packets, queries, Session};
use async_trait::async_trait;
use oxide_core::{
    net::{Connection, Events, Packet},
    Db, Redis,
};

pub struct LoginServerEventHandler {
    db: Db,
    redis: Redis,
}

impl LoginServerEventHandler {
    pub fn new(db: Db, redis: Redis) -> Self {
        Self { db, redis }
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

        if let Err(e) = Session::create(connection.session_id, &self.redis).await {
            log::error!("Error creating session {}: {}", connection.session_id, e);
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
            .handle(connection, self.db.clone(), self.redis.clone())
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

        let session = match Session::load(connection.session_id, &self.redis).await {
            Ok(session) => session,
            Err(e) => {
                log::error!("Error loading session {}: {}", connection.session_id, e);
                return;
            }
        };

        if let Err(e) = queries::update_login_state(session.account_id, 0, &self.db).await {
            log::error!("On disconnect error: {}", e);
        }

        if let Err(e) = Session::delete(connection.session_id, &self.redis).await {
            log::error!("Error deleting session {}: {}", connection.session_id, e);
        }
    }
}
