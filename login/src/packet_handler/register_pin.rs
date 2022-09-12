use crate::{client::Client, packets, queries};
use oxide_core::{net::Packet, Db, Result};

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

    pub async fn handle(self, client: &mut Client, db: Db) -> Result<()> {
        if self.flag == 0 {
            client.disconnect().await?;
            return Ok(());
        }

        client.session.pin = self.pin.clone();

        queries::update_pin(client.session.account_id, &self.pin, &db).await?;
        queries::update_login_state(client.session.account_id, 0, &db).await?;
        let packet = packets::pin_registered();
        client.send(packet).await?;
        Ok(())
    }
}
