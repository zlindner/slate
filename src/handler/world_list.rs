use crate::{client::Client, Result, login::packets};

#[derive(Debug)]
pub struct WorldList;

impl WorldList {
    pub fn new() -> Self {
        Self
    }

    pub async fn handle(self, client: &mut Client) -> Result<()> {
        let shared = &client.shared;
        let connection = &mut client.connection;

        for world in shared.worlds.iter() {
            connection.write_packet(packets::world_details(&world)).await?;
        }

        // tell the client that we have sent details for all available worlds
        connection.write_packet(packets::world_list_end()).await?;

        // TODO select_world?
        // TODO recommended_world?

        Ok(())
    }
}
