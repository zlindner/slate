use crate::packets;
use deadpool_redis::redis::AsyncCommands;
use oxide_core::{
    maple::{Character, Skill},
    net::{Connection, Packet},
    Db, Redis, Result,
};

pub struct Connect {
    session_id: i32,
}

impl Connect {
    pub fn new(mut packet: Packet) -> Self {
        Self {
            session_id: packet.read_int(),
        }
    }

    pub async fn handle(self, connection: &mut Connection, db: Db, redis: Redis) -> Result<()> {
        let mut redis = redis.get().await?;
        let key = format!("login_session:{}", self.session_id);
        let character_id: i32 = redis.hget(&key, "character_id").await?;
        let account_id: i32 = redis.hget(&key, "account_id").await?;

        let mut character: Character = sqlx::query_as(
            "SELECT * \
            FROM characters \
            WHERE id = $1 AND account_id = $2 AND world_id = $3",
        )
        .bind(character_id)
        .bind(account_id)
        .bind(0) // FIXME pass in world id
        .fetch_one(&db)
        .await?;

        // TODO these are essentially "skill entries", we need to match these up with data loaded
        // from wz files... or do we
        let skills: Vec<Skill> = sqlx::query_as(
            "SELECT * \
            FROM skills \
            WHERE character_id = $1",
        )
        .bind(character_id)
        .fetch_all(&db)
        .await?;

        character.skills = skills;

        connection
            .write_packet(packets::character_info(character))
            .await?;

        Ok(())
    }
}
