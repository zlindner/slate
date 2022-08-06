use crate::{packets, state::State};
use oxide_core::{net::Connection, Result};
use std::sync::Arc;

pub struct WorldList;

impl WorldList {
    pub fn new() -> Self {
        Self
    }

    pub async fn handle(self, connection: &mut Connection, state: Arc<State>) -> Result<()> {
        /*for world in shared.worlds.iter() {
            connection
                .write_packet(packets::world_details(&world))
                .await?;
        }*/

        // FIXME
        connection
            .write_packet(packets::world_details_temp())
            .await?;

        // tell the client that we have sent details for all available worlds
        connection.write_packet(packets::world_list_end()).await?;

        // pre-select world with id "0" for the client
        // TODO this should be the most active world, not really a priority to fix
        connection.write_packet(packets::world_select(0)).await?;

        // add the recommended world text for each world
        connection
            .write_packet(packets::view_recommended_temp())
            .await?;

        Ok(())
    }
}
