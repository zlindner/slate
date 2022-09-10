use crate::{packets, queries};
use deadpool_redis::redis::AsyncCommands;
use oxide_core::{
    net::{Connection, Packet},
    Db, Redis, Result,
};

pub struct RegisterPin {
    flag: u8,
    pin: String,
}

impl RegisterPin {
    pub fn new(mut packet: Packet) -> Self {
        let flag = packet.read_byte();
        let pin = match flag {
            0 => "".to_string(),
            _ => packet.read_string(),
        };

        Self { flag, pin }
    }

    pub async fn handle(self, connection: &mut Connection, db: Db, redis: Redis) -> Result<()> {
        if self.flag == 0 {
            connection.close().await?;
            return Ok(());
        }

        let mut redis = redis.get().await?;
        let key = format!("login_session:{}", connection.session_id);
        let account_id: i32 = redis.hget(&key, "account_id").await?;
        redis.hset(&key, "pin", &self.pin).await?;

        queries::update_pin(account_id, &self.pin, &db).await?;
        queries::update_login_state(account_id, 0, &db).await?;
        connection.write_packet(packets::pin_registered()).await?;

        Ok(())
    }
}
