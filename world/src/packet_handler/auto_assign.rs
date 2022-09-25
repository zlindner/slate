use crate::{client::Client, packets};
use oxide_core::{net::Packet, Db, Result};

pub struct AutoAssign;

impl AutoAssign {
    pub fn new(packet: Packet) -> Self {
        Self
    }

    pub async fn handle(self, client: &mut Client, db: Db) -> Result<()> {
        // TODO

        let packet = packets::enable_actions();
        client.send(packet).await?;

        Ok(())
    }
}
