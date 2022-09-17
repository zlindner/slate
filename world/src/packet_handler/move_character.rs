use crate::client::Client;
use oxide_core::{net::Packet, Db, Result};

pub struct MoveCharacter {
    packet: Packet,
}

impl MoveCharacter {
    pub fn new(packet: Packet) -> Self {
        MoveCharacter { packet }
    }

    pub async fn handle(mut self, client: &mut Client, db: Db) -> Result<()> {
        if self.packet.remaining() < 1 {
            log::debug!("Received empty movement packet");
            return Ok(());
        }

        if client.character.is_none() {
            return Ok(());
        }

        let mut character = client.character.as_mut().unwrap();
        let commands = self.packet.read_byte();

        for _ in 0..commands {
            let command = self.packet.read_byte();

            match command {
                0 | 5 | 17 => {
                    let x = self.packet.read_short();
                    let y = self.packet.read_short();
                    character.pos = (x.into(), y.into());
                    self.packet.skip(6);
                    let stance = self.packet.read_byte();
                    let duration = self.packet.read_short();
                }
                _ => {
                    log::debug!("Unhandled movement command: {}", command);
                }
            };
        }

        log::debug!("Character pos: {:?}", character.pos);
        Ok(())
    }
}
