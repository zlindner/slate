use crate::packets;
use oxide_core::{
    net::{Connection, Packet},
    state::Session,
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
        let mut session = Session::load(connection.session_id, &redis).await?;
        log::info!("char id: {}", self.character_id);
        session.character_id = self.character_id;
        // TODO save mac and host addrs, validate on world server?
        session.save(&redis).await?;

        connection
            .write_packet(packets::channel_server_ip(connection.session_id))
            .await?;

        Ok(())
    }
}
