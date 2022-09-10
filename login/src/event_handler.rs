use crate::{packet_handler::LoginServerPacketHandler, queries};
use async_trait::async_trait;
use deadpool_redis::redis::{cmd, AsyncCommands, RedisResult};
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

    async fn on_shutdown(&self) {
        log::info!("Login server shutting down...");

        if let Err(e) = queries::logout_all(&self.db).await {
            log::error!("Error executing logout_all: {}", e);
        }

        // clear redis cache
        let mut state = self.redis.get().await.unwrap();
        let _: RedisResult<()> = cmd("FLUSHDB").query_async(&mut state).await;
    }

    async fn on_connect(&self, connection: &mut Connection) {
        log::info!(
            "Client connected to login server (session {})",
            connection.session_id
        );
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

        // TODO error handle
        let mut redis = self.redis.get().await.unwrap();
        let key = format!("login_session:{}", connection.session_id);
        let account_id: i32 = redis.hget(&key, "account_id").await.unwrap();

        if let Err(e) = queries::update_login_state(account_id, 0, &self.db).await {
            log::error!("Error updating login state: {}", e);
        }

        // type is needed for some reason here? rust pls
        let _: String = redis.del(&key).await.unwrap();
    }
}
