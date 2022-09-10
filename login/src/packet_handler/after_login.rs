use crate::packets::{self, PinOperation};
use deadpool_redis::redis::AsyncCommands;
use oxide_core::{
    net::{Connection, Packet},
    Redis, Result,
};

pub struct AfterLogin {
    a: u8,
    b: u8,
    pin: Option<String>,
}

impl AfterLogin {
    pub fn new(mut packet: Packet) -> Self {
        let a = packet.read_byte();

        let b = match packet.remaining() {
            0 => 5,
            _ => packet.read_byte(),
        };

        let pin = match b {
            0 => Some(packet.read_string()),
            _ => None,
        };

        Self { a, b, pin }
    }

    pub async fn handle(self, connection: &mut Connection, redis: Redis) -> Result<()> {
        let mut redis = redis.get().await?;
        let key = format!("login_session:{}", connection.session_id);
        let pin: String = redis.hget(&key, "pin").await?;

        let op = match (self.a, self.b) {
            (1, 1) => match pin.is_empty() {
                true => PinOperation::Register,
                false => PinOperation::Request,
            },
            (1, 0) | (2, 0) => {
                let pin_attempts: u8 = redis.hget(&key, "pin_attempts").await?;

                if pin_attempts >= 6 {
                    connection.close().await?;
                    return Ok(());
                }

                if !pin.is_empty() && self.pin.unwrap() == pin {
                    redis.hset(&key, "pin_attempts", 0).await?;

                    if self.a == 1 {
                        PinOperation::Accepted
                    } else {
                        PinOperation::Register
                    }
                } else {
                    redis.hset(&key, "pin_attempts", pin_attempts + 1).await?;
                    PinOperation::RequestAfterFailure
                }
            }
            _ => {
                connection.close().await?;
                return Ok(());
            }
        };

        connection.write_packet(packets::pin_operation(op)).await?;
        Ok(())
    }
}
