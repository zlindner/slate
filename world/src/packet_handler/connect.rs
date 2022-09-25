use crate::{client::Client, packets};
use oxide_core::{
    maple::{Character, Skill},
    net::Packet,
    pg::{PgCharacter, PgKeymap, PgSession},
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
        let session: PgSession = sqlx::query_as("SELECT * FROM sessions WHERE id = $1")
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

        let keymaps: Vec<PgKeymap> =
            sqlx::query_as("SELECT * FROM keymaps WHERE character_id = $1")
                .bind(session.character_id)
                .fetch_all(&db)
                .await?;

        let mut character = Character::new();
        character.channel_id = session.channel_id;
        character.pg = pg_character;
        character.skills = skills;
        character.keymaps = keymaps;

        client.character = Some(character);

        let packet = packets::character_info(&client.character.as_ref().unwrap());
        client.send(packet).await?;

        let packet = packets::keymap(&client.character.as_ref().unwrap());
        client.send(packet).await?;

        let packet = packets::quickmap();
        client.send(packet).await?;

        let packet = packets::macros();
        client.send(packet).await?;

        let packet = packets::buddy_list();
        client.send(packet).await?;

        let packet = packets::family_entitlements();
        client.send(packet).await?;

        let packet = packets::family_info();
        client.send(packet).await?;

        // TODO load guild
        // TODO show notes

        let packet = packets::gender(&client.character.as_ref().unwrap());
        client.send(packet).await?;

        let packet = packets::enable_report();
        client.send(packet).await?;

        // TODO should we delete the session from db here?

        Ok(())
    }
}
