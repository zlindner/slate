use crate::{
    character::Character,
    client::Client,
    login::{packets, queries},
    net::packet::Packet,
    world::CapacityStatus,
};
use oxide_core::Result;

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

    pub async fn handle(self, client: &mut Client) -> Result<()> {
        let shared = &client.shared;
        let db = &client.db;
        let connection = &mut client.connection;

        let world = match shared.worlds.get(self.world_id as usize) {
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
        client.channel_id = Some(channel.id);

        let rows = match queries::get_characters(client.id.unwrap(), world.config.id, db).await {
            Ok(characters) => characters,
            Err(_) => Vec::new(),
        };

        let mut characters: Vec<Character> = Vec::new();

        for row in rows.iter() {
            characters.push(Character::from_row(row));
        }

        connection
            .write_packet(packets::character_list(&characters))
            .await?;

        Ok(())
    }
}
