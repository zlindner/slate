use crate::{
    client::Client,
    login::{packets, queries},
};
use once_cell::sync::Lazy;
use oxide_core::{net::Packet, Result};
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

    pub async fn handle(self, client: &mut Client) -> Result<()> {
        let valid = Self::is_valid_name(&self.name, client).await?;
        client
            .connection
            .write_packet(packets::character_name(&self.name, valid))
            .await?;
        Ok(())
    }

    async fn is_valid_name(name: &String, client: &mut Client) -> Result<bool> {
        // TODO check if name is blacklisted

        // check if name already exists
        let id = queries::get_character_id_by_name(name, &client.db).await?;

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
