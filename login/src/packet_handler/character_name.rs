use crate::packets;
use once_cell::sync::Lazy;
use oxide_core::{
    net::{Connection, Packet},
    Db, Result,
};
use regex::Regex;

static VALID_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"[a-zA-Z0-9]{3,12}").unwrap());

pub struct CharacterName {
    name: String,
}

impl CharacterName {
    pub fn new(mut packet: Packet) -> Self {
        Self {
            name: packet.read_string(),
        }
    }

    pub async fn handle(self, connection: &mut Connection, db: Db) -> Result<()> {
        let is_valid = Self::is_valid_name(&self.name, &db).await?;

        connection
            .write_packet(packets::character_name(&self.name, is_valid))
            .await?;

        Ok(())
    }

    async fn is_valid_name(name: &String, db: &Db) -> Result<bool> {
        // TODO check if name is blacklisted

        // check if name already exists
        let id = sqlx::query(
            "SELECT id \
            FROM characters \
            WHERE name = $1",
        )
        .bind(name)
        .fetch_optional(db)
        .await?;

        if id.is_some() {
            return Ok(false);
        }

        // check if name matches regex
        if !VALID_REGEX.is_match(name) {
            return Ok(false);
        }

        Ok(true)
    }
}
