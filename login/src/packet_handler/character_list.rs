use crate::packets;
use deadpool_redis::redis::AsyncCommands;
use oxide_core::{
    maple::Character,
    net::{Connection, Packet},
    Db, Redis, Result,
};

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

    pub async fn handle(self, connection: &mut Connection, db: Db, redis: Redis) -> Result<()> {
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

        let mut redis = redis.get().await?;
        let key = format!("login_session:{}", connection.session_id);
        let account_id: i32 = redis.hget(&key, "account_id").await?;

        // TODO pass world id in
        let characters: Vec<Character> = sqlx::query_as(
            "SELECT * \
            FROM characters \
            WHERE account_id = $1 AND world_id = $2",
        )
        .bind(account_id)
        .bind(self.world_id as i32)
        .fetch_all(&db)
        .await?;

        redis
            .hset_multiple(
                key,
                &[("world_id", self.world_id), ("channel_id", self.channel_id)],
            )
            .await?;

        connection
            .write_packet(packets::character_list(&characters))
            .await?;

        Ok(())
    }
}
