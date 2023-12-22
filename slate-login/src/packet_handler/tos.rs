use crate::server::LoginSession;
use slate_data::sql::{self, account::LoginState};
use slate_net::Packet;

/// Login server: accept tos packet (0x07)
/// Called after client successfully logs in, but hasn't accepted tos
pub async fn handle(mut packet: Packet, session: &mut LoginSession) -> anyhow::Result<()> {
    let accepted = packet.read_byte();

    // If the tos isn't accepted, client returns 0 and disconnects itself
    if accepted != 1 {
        return Ok(());
    }

    let account = sql::Account::load_optional_by_id(session.data.account_id, &session.db).await?;

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

    session.data.account_id = account.id;
    session.data.pin = account.pin.clone();
    session.data.pic = account.pic.clone();

    sql::Account::update_tos(session.data.account_id, true, &session.db).await?;
    sql::Account::update_login_state(session.data.account_id, LoginState::LoggedIn, &session.db)
        .await?;

    session
        .stream
        .write_packet(super::login::login_succeeded(session, &account))
        .await?;

    Ok(())
}
