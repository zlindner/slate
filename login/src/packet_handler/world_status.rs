use crate::{client::Client, login::packets, world::CapacityStatus};
use oxide_core::{net::Packet, Result};

pub struct WorldStatus {
    world_id: i16,
}

impl WorldStatus {
    pub fn new(mut packet: Packet) -> Self {
        let world_id = packet.read_short();

        Self { world_id }
    }

    pub async fn handle(self, client: &mut Client) -> Result<()> {
        let shared = &client.shared;
        let connection = &mut client.connection;

        let world = shared.worlds.get(self.world_id as usize);

        let capacity_status = match world {
            Some(world) => world.get_capacity_status(),
            None => CapacityStatus::Full,
        };

        connection
            .write_packet(packets::world_status(capacity_status))
            .await?;

        Ok(())
    }
}
