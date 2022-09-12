use crate::{
    client::Client,
    packets::{self, PinOperation},
};
use oxide_core::{net::Packet, Result};

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

    pub async fn handle(self, client: &mut Client) -> Result<()> {
        let op = match (self.a, self.b) {
            (1, 1) => match client.session.pin.is_empty() {
                true => PinOperation::Register,
                false => PinOperation::Request,
            },
            (1, 0) | (2, 0) => {
                if client.session.pin_attempts >= 6 {
                    client.disconnect().await?;
                    return Ok(());
                }

                client.session.pin_attempts += 1;

                if !client.session.pin.is_empty() && self.pin.unwrap() == client.session.pin {
                    client.session.pin_attempts = 0;

                    if self.a == 1 {
                        PinOperation::Accepted
                    } else {
                        PinOperation::Register
                    }
                } else {
                    PinOperation::RequestAfterFailure
                }
            }
            _ => {
                client.disconnect().await?;
                return Ok(());
            }
        };

        let packet = packets::pin_operation(op);
        client.send(packet).await?;
        Ok(())
    }
}
