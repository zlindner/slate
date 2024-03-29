use crate::server::LoginSession;
use slate_net::Packet;
use sqlx::Row;

/// Login server: validate character name packet (0x15)
/// TODO
pub async fn handle(mut packet: Packet, session: &mut LoginSession) -> anyhow::Result<()> {
    let name = packet.read_string();

    // TODO add blocked names list (rot13?)

    // Check if the name is already taken is current world
    let num_characters: i32 =
        sqlx::query("SELECT COUNT(*) as count FROM characters WHERE name = ? AND world_id = ?")
            .bind(name.clone())
            .bind(session.data.world_id)
            .fetch_one(&session.db)
            .await?
            .get("count");

    if num_characters >= 1 {
        return session.stream.write_packet(valid_name(name, false)).await;
    }

    // Name has to be alphanumeric between 3 and 12 characters long
    if !name.chars().all(char::is_alphanumeric) || name.len() < 3 || name.len() > 12 {
        return session.stream.write_packet(valid_name(name, false)).await;
    }

    session.stream.write_packet(valid_name(name, true)).await?;
    Ok(())
}

/// Packet indicating if the selected character name is valid or not
fn valid_name(name: String, valid: bool) -> Packet {
    let mut packet = Packet::new(0x0D);
    packet.write_string(&name);
    packet.write_byte(!valid as u8);
    packet
}
