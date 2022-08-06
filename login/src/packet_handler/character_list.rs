use crate::{packets, State};
use oxide_core::{
    net::{Connection, Packet},
    Character, Db, Result,
};
use std::sync::Arc;

pub struct CharacterList {
    world_id: u8,
    channel_id: u8,
}

impl CharacterList {
    pub fn new(mut packet: Packet) -> Self {
        packet.skip(1);

        let world_id = packet.read_byte();
        let channel_id = packet.read_byte();

        Self {
            world_id,
            channel_id,
        }
    }

    pub async fn handle(
        self,
        connection: &mut Connection,
        db: &Db,
        state: Arc<State>,
    ) -> Result<()> {
        /*let world = match shared.worlds.get(self.world_id as usize) {
            Some(world) => world,
            None => {
                connection
                    .write_packet(packets::world_status(CapacityStatus::Full))
                    .await?;
                return Ok(());
            }
        };

        if world.get_capacity_status() == CapacityStatus::Full {
            connection
                .write_packet(packets::world_status(CapacityStatus::Full))
                .await?;
            return Ok(());
        }

        let channel = match world.channels.get(self.channel_id as usize) {
            Some(channel) => channel,
            None => {
                connection
                    .write_packet(packets::world_status(CapacityStatus::Full))
                    .await?;
                return Ok(());
            }
        };

        client.world_id = Some(world.config.id);
        client.channel_id = Some(channel.id);*/

        let session = state.sessions.get(&connection.session_id).unwrap();

        // TODO pass world id in
        let characters: Vec<Character> = sqlx::query_as(
            "SELECT * \
            FROM characters \
            WHERE account_id = $1 AND world_id = $2",
        )
        .bind(session.account_id)
        .bind(0) // FIXME pass in world id, can we just use self.world_id here?
        .fetch_all(db)
        .await?;

        connection
            .write_packet(packets::character_list(&characters))
            .await?;

        Ok(())
    }
}
