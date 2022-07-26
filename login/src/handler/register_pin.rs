use crate::{
    client::Client,
    login::{packets, queries},
    net::packet::Packet,
    Result,
};

pub struct RegisterPin {
    flag: u8,
    packet: Packet,
}

impl RegisterPin {
    pub fn new(mut packet: Packet) -> Self {
        let flag = packet.read_byte();

        Self { flag, packet }
    }

    pub async fn handle(mut self, client: &mut Client) -> Result<()> {
        let db = &client.db;
        let connection = &mut client.connection;

        if self.flag == 0 {
            queries::update_login_state(client.id.unwrap(), 0, db).await?;
            return Ok(());
        }

        let pin = self.packet.read_string();
        client.pin = Some(pin.clone());
        queries::update_pin(client.id.unwrap(), &pin, db).await?;

        connection.write_packet(packets::pin_registered()).await?;
        queries::update_login_state(client.id.unwrap(), 0, db).await?;

        Ok(())
    }
}
