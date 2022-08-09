use crate::packets;
use oxide_core::{
    net::{Connection, Packet},
    state::Session,
    Character, Db, Redis, Result,
};

pub struct Connect {
    character_id: i32,
}

impl Connect {
    pub fn new(mut packet: Packet) -> Self {
        Self {
            character_id: packet.read_int(),
        }
    }

    pub async fn handle(self, connection: &mut Connection, db: Db, redis: Redis) -> Result<()> {
        let session = Session::load(connection.session_id, &redis).await?;

        let character: Character = sqlx::query_as(
            "SELECT * \
            FROM characters \
            WHERE account_id = $1 AND world_id = $2",
        )
        .bind(session.account_id)
        .bind(0) // FIXME pass in world id
        .fetch_one(&db)
        .await?;

        connection
            .write_packet(packets::character_info(character))
            .await?;

        Ok(())
    }
}
