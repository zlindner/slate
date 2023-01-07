use crate::client::LoginClient;
use anyhow::Result;
use oxy_core::{net::Packet, prisma::character};

/// Login server: validate character name packet (0x15)
///
pub async fn handle(mut packet: Packet, client: &mut LoginClient) -> Result<()> {
    let name = packet.read_string();

    // TODO add blocked names list (rot13?)

    // Check if the name is already taken
    let character = client
        .db
        .character()
        .find_unique(character::name::equals(name.clone()))
        .exec()
        .await?;

    if character.is_some() {
        let response = valid_name(name, false);
        return client.send(response).await;
    }

    // Name has to be alphanumeric between 3 and 12 characters long
    if !name.chars().all(char::is_alphanumeric) || name.len() < 3 || name.len() > 12 {
        let response = valid_name(name, false);
        return client.send(response).await;
    }

    let response = valid_name(name, true);
    client.send(response).await?;
    Ok(())
}

///
fn valid_name(name: String, valid: bool) -> Packet {
    let mut packet = Packet::new();
    packet.write_short(0x0D);
    packet.write_string(&name);
    packet.write_byte(!valid as u8);
    packet
}
