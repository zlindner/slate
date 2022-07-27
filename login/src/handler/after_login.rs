use crate::{
    client::Client,
    login::{
        packets::{self, PinOperation},
        queries,
    },
};
use oxide_core::{net::Packet, Result};

pub struct AfterLogin {
    a: u8,
    b: u8,
    packet: Packet,
}

impl AfterLogin {
    pub fn new(mut packet: Packet) -> Self {
        let a = packet.read_byte();

        let b = match packet.remaining() {
            0 => 5,
            _ => packet.read_byte(),
        };

        Self { a, b, packet }
    }

    pub async fn handle(mut self, client: &mut Client) -> Result<()> {
        let db = &client.db;

        let op = match (self.a, self.b) {
            (1, 1) => match client.pin {
                None => Some(PinOperation::Register),
                Some(_) => Some(PinOperation::Request),
            },
            (1, 0) => {
                let pin = self.packet.read_string();
                Self::validate_pin(client, pin, 1).await?
            }
            (2, 0) => {
                let pin = self.packet.read_string();
                Self::validate_pin(client, pin, 2).await?
            }
            _ => {
                // TODO can possibly send PinOperation::ConnectionFailed here?
                queries::update_login_state(client.id.unwrap(), 0, db).await?;
                None
            }
        };

        if op.is_some() {
            client
                .connection
                .write_packet(packets::pin_operation(op.unwrap()))
                .await?;
        }

        Ok(())
    }

    async fn validate_pin(
        client: &mut Client,
        pin: String,
        flag: u8,
    ) -> Result<Option<PinOperation>> {
        client.pin_attempts += 1;

        if client.pin_attempts >= 6 {
            client.disconnect().await?;
            return Ok(None);
        }

        if client.pin.is_some() && &pin == client.pin.as_ref().unwrap() {
            client.pin_attempts = 0;

            if flag == 1 {
                return Ok(Some(PinOperation::Accepted));
            } else {
                return Ok(Some(PinOperation::Register));
            }
        }

        Ok(Some(PinOperation::RequestAfterFailure))
    }
}
