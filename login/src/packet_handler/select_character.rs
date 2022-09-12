use crate::{client::Client, packets};
use oxide_core::{net::Packet, Db, Result};

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

    pub async fn handle(self, client: &mut Client, db: Db) -> Result<()> {
        client.session.character_id = self.character_id;

        // TODO save mac and host addrs, validate on world server?
        // TODO load world_id and channel_id from session

        let packet = packets::channel_server_ip(client.session.id);
        client.send(packet).await?;
        Ok(())
    }
}
