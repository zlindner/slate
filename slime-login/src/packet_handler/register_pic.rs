use crate::server::LoginSession;
use slime_net::Packet;

/// Login server: register pic packet (0x1D)
/// TODO
pub async fn handle(mut packet: Packet, session: &mut LoginSession) -> anyhow::Result<()> {
    if packet.read_byte() == 0 {
        session.stream.close().await?;
        return Ok(());
    }

    let character_id = packet.read_int();
    let mac_addr = packet.read_string();
    let host_addr = packet.read_string();
    // TODO we can check mac_addr/hwid from host_addr if we want to prevent multi-logging

    let pic = packet.read_string();

    if pic.is_empty() {
        session.stream.close().await?;
        return Ok(());
    }

    // Set account's pic column
    sqlx::query("UPDATE accounts SET pic = ? WHERE id = ?")
        .bind(pic.clone())
        .bind(session.data.account_id)
        .execute(&session.db)
        .await?;

    session.data.pic = pic;
    session.data.character_id = character_id;

    super::select_character::connect_to_world_server(session).await?;
    Ok(())
}
