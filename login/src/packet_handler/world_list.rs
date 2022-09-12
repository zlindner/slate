use crate::{client::Client, packets};
use oxide_core::Result;

pub struct WorldList;

impl WorldList {
    pub fn new() -> Self {
        Self
    }

    pub async fn handle(self, client: &mut Client) -> Result<()> {
        /*for world in shared.worlds.iter() {
            connection
                .write_packet(packets::world_details(&world))
                .await?;
        }*/

        // FIXME
        let packet = packets::world_details_temp();
        client.send(packet).await?;

        // tell the client that we have sent details for all available worlds
        let packet = packets::world_list_end();
        client.send(packet).await?;

        // pre-select world with id "0" for the client
        // TODO this should be the most active world, not really a priority to fix
        let packet = packets::world_select(0);
        client.send(packet).await?;

        // add the recommended world text for each world
        let packet = packets::view_recommended_temp();
        client.send(packet).await?;
        Ok(())
    }
}
