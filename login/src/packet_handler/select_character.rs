use crate::packets;
use deadpool_redis::redis::AsyncCommands;
use oxide_core::{
    net::{Connection, Packet},
    Db, Redis, Result,
};

pub struct SelectCharacter {
    character_id: i32,
    mac_addr: String,
    host_addr: String,
}

impl SelectCharacter {
    pub fn new(mut packet: Packet) -> Self {
        Self {
            character_id: packet.read_int(),
            mac_addr: packet.read_string(),
            host_addr: packet.read_string(),
        }
    }

    pub async fn handle(self, connection: &mut Connection, db: Db, redis: Redis) -> Result<()> {
        let mut redis = redis.get().await?;
        let key = format!("login_session:{}", connection.session_id);
        redis.hset(&key, "character_id", self.character_id).await?;

        let world_id: String = redis.hget(&key, "world_id").await?;
        let channel_id: String = redis.hget(&key, "channel_id").await?;

        // TODO save mac and host addrs, validate on world server?
        // TODO load world_id and channel_id from session

        connection
            .write_packet(packets::channel_server_ip(connection.session_id))
            .await?;

        Ok(())
    }
}
