use crate::packets;
use oxide_core::{
    net::{Connection, Packet},
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
        connection
            .write_packet(packets::channel_server_ip(self.character_id))
            .await?;

        Ok(())
    }
}
