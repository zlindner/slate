use crate::{packets, state::State};
use oxide_core::{
    net::{Connection, Packet},
    Result,
};
use std::sync::Arc;

pub struct WorldStatus {
    world_id: i16,
}

impl WorldStatus {
    pub fn new(mut packet: Packet) -> Self {
        let world_id = packet.read_short();

        Self { world_id }
    }

    pub async fn handle(self, connection: &mut Connection, state: Arc<State>) -> Result<()> {
        /*let world = shared.worlds.get(self.world_id as usize);

        let capacity_status = match world {
            Some(world) => world.get_capacity_status(),
            None => CapacityStatus::Full,
        };*/

        connection
            .write_packet(packets::world_status_temp())
            .await?;

        Ok(())
    }
}
