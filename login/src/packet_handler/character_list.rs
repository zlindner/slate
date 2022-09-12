use crate::{client::Client, packets};
use oxide_core::{maple::Character, net::Packet, Db, Result};

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

    pub async fn handle(self, client: &mut Client, db: Db) -> Result<()> {
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

        // TODO pass world id in
        let characters: Vec<Character> = sqlx::query_as(
            "SELECT * \
            FROM characters \
            WHERE account_id = $1 AND world_id = $2",
        )
        .bind(client.session.account_id)
        .bind(self.world_id as i32)
        .fetch_all(&db)
        .await?;

        client.session.world_id = self.world_id as i16;
        client.session.channel_id = self.channel_id as i16;

        let packet = packets::character_list(&characters);
        client.send(packet).await?;
        Ok(())
    }
}
