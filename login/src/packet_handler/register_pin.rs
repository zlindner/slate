use crate::{packets, queries, Session};
use oxide_core::{
    net::{Connection, Packet},
    Db, Redis, Result,
};

pub struct RegisterPin {
    flag: u8,
    pin: Option<String>,
}

impl RegisterPin {
    pub fn new(mut packet: Packet) -> Self {
        let flag = packet.read_byte();
        let pin = match flag {
            0 => None,
            _ => Some(packet.read_string()),
        };

        Self { flag, pin }
    }

    pub async fn handle(self, connection: &mut Connection, db: Db, redis: Redis) -> Result<()> {
        if self.flag == 0 {
            connection.close().await?;
            return Ok(());
        }

        let mut session = Session::get(connection.session_id, redis).await?;

        session.pin = self.pin.clone();
        queries::update_pin(session.account_id, &self.pin.unwrap(), &db).await?;

        connection.write_packet(packets::pin_registered()).await?;
        queries::update_login_state(session.account_id, 0, &db).await?;
        Ok(())
    }
}
