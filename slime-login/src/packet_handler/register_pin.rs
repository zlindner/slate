use crate::{model::LoginState, query, server::LoginSession};
use slime_net::Packet;

/// Login server: register pin packet (0x0A)
/// TODO
pub async fn handle(mut packet: Packet, session: &mut LoginSession) -> anyhow::Result<()> {
    if packet.read_byte() == 0 {
        session.stream.close().await?;
        return Ok(());
    }

    let pin = packet.read_string();

    if pin.is_empty() {
        session.stream.close().await?;
        return Ok(());
    }

    // Set account's pin column
    sqlx::query("UPDATE accounts SET pin = ? WHERE id = ?")
        .bind(pin.clone())
        .bind(session.data.account_id)
        .execute(&session.db)
        .await?;

    session.data.pin = pin;
    query::update_login_state(session, LoginState::LoggedOut).await?;
    session.stream.write_packet(pin_registered()).await?;

    Ok(())
}

/// Packet indicating the pin was succesfully registered
fn pin_registered() -> Packet {
    let mut packet = Packet::new(0x07);
    packet.write_byte(0);
    packet
}
