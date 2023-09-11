use crate::{
    model::{Account, LoginState},
    query,
    server::LoginSession,
};
use slime_net::Packet;

/// Login server: accept tos packet (0x07)
/// Called after client successfully logs in, but hasn't accepted tos
pub async fn handle(mut packet: Packet, session: &mut LoginSession) -> anyhow::Result<()> {
    let accepted = packet.read_byte();

    // If the tos isn't accepted, client returns 0 and disconnects itself
    if accepted != 1 {
        return Ok(());
    }

    let account = sqlx::query_as::<_, Account>("SELECT * FROM accounts WHERE id = ?")
        .bind(session.data.account_id)
        .fetch_optional(&session.db)
        .await?;

    let account = match account {
        Some(account) => account,
        None => {
            log::error!("Attempted to accept TOS for account that doesn't exist");
            return Ok(());
        }
    };

    if !matches!(account.state, LoginState::LoggedOut) {
        log::error!("Account is already logged in");
        return Ok(());
    }

    // Set account's accepted_tos column to true
    sqlx::query("UPDATE accounts SET accepted_tos = ? WHERE id = ?")
        .bind(true)
        .bind(session.data.account_id)
        .execute(&session.db)
        .await?;

    session.data.account_id = account.id;
    session.data.pin = account.pin.clone();
    session.data.pic = account.pic.clone();
    query::update_login_state(&session, LoginState::LoggedIn).await?;

    session
        .stream
        .write_packet(super::login::login_succeeded(&session, &account))
        .await?;

    Ok(())
}
