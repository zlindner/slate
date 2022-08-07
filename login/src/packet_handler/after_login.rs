use crate::{
    packets::{self, PinOperation},
    Session,
};
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
        let mut session = Session::get(connection.session_id, redis).await?;

        let op = match (self.a, self.b) {
            (1, 1) => match session.pin {
                None => PinOperation::Register,
                Some(_) => PinOperation::Request,
            },
            (1, 0) | (2, 0) => {
                if session.pin_attempts >= 6 {
                    connection.close().await?;
                    return Ok(());
                }

                session.pin_attempts += 1;

                if session.pin.is_some() && &self.pin.unwrap() == session.pin.as_ref().unwrap() {
                    session.pin_attempts = 0;

                    if self.a == 1 {
                        PinOperation::Accepted
                    } else {
                        PinOperation::Register
                    };
                }

                PinOperation::RequestAfterFailure
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
