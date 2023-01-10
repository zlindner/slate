use crate::{client::LoginClient, shared::Shared};
use anyhow::Result;
use oxy_core::{
    net::Packet,
    prisma::{self, LoginState},
};

/// Login server: accept tos packet (0x07)
/// Called after client successfully logs in, but hasn't yet accepted tos
pub async fn handle(mut packet: Packet, client: &mut LoginClient, shared: &Shared) -> Result<()> {
    let accepted = packet.read_byte();

    // If the tos isn't accepted, client returns 0 and disconnects itself
    if accepted != 1 {
        return Ok(());
    }

    // Even though we load the account in login::handle, we should confirm that
    // the account's state hasn't changed (and makes handling a bit nicer)
    let account = match client.get_account().await? {
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

    // Update account's tos in db
    client
        .db
        .account()
        .update(
            prisma::account::id::equals(account.id),
            vec![prisma::account::tos::set(true)],
        )
        .exec()
        .await?;

    client.session.account_id = account.id;
    client.session.pin = account.pin.clone();
    client.session.pic = account.pic.clone();
    client.session.tos = account.tos;
    client.update_login_state(LoginState::LoggedIn).await?;

    let response = super::login::login_succeeded(&account, client, &shared.config);
    client.send(response).await?;
    Ok(())
}
