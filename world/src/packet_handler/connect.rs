use crate::{client::Client, packets};
use oxide_core::{
    maple::{Character, Skill},
    net::Packet,
    pg::{PgCharacter, Session},
    Db, Result,
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

    pub async fn handle(self, client: &mut Client, db: Db) -> Result<()> {
        let session: Session = sqlx::query_as("SELECT * FROM sessions WHERE id = $1")
            .bind(self.session_id)
            .fetch_one(&db)
            .await?;

        let pg_character: PgCharacter = sqlx::query_as(
            "SELECT * FROM characters WHERE id = $1 AND account_id = $2 AND world_id = $3",
        )
        .bind(session.character_id)
        .bind(session.account_id)
        .bind(session.world_id)
        .fetch_one(&db)
        .await?;

        // TODO these are essentially "skill entries", we need to match these up with data loaded
        // from wz files... or do we
        let skills: Vec<Skill> = sqlx::query_as("SELECT * FROM skills WHERE character_id = $1")
            .bind(session.character_id)
            .fetch_all(&db)
            .await?;

        let mut character = Character::new();
        character.pg = pg_character;
        character.skills = skills;
        client.character = Some(character);

        client
            .send(packets::character_info(&client.character.as_ref().unwrap()))
            .await?;

        // TODO should we delete the session from db here?

        Ok(())
    }
}
