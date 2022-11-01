use super::Config;
use anyhow::Result;
use oxy_core::{
    net::{Client, Packet},
    prisma::account,
    prisma::LoginState,
};

/// Login server: login packet (0x01)
/// Called when the client clicks login after entering name and password
pub async fn handle(mut packet: Packet, client: &mut Client, config: &Config) -> Result<()> {
    if client.session.login_attempts >= 5 {
        let response = login_failed(LoginError::TooManyAttempts);
        return client.send(response).await;
    }

    client.session.login_attempts += 1;

    let name = packet.read_string();
    let account = client
        .db
        .account()
        .find_unique(account::name::equals(name))
        .exec()
        .await?;

    let account = match account {
        Some(account) => account,
        None => {
            let response = login_failed(LoginError::AccountNotFound);
            return client.send(response).await;
        }
    };

    let password = packet.read_string();

    // Validate the bytes of the entered password against the hash stored in db
    if argon2::verify_encoded(&account.password, password.as_bytes()).is_err() {
        let response = login_failed(LoginError::IncorrectPassword);
        return client.send(response).await;
    }

    if !matches!(account.state, LoginState::LoggedOut) {
        let response = login_failed(LoginError::AlreadyLoggedIn);
        return client.send(response).await;
    }

    if account.banned == true {
        let response = login_failed(LoginError::AccountBanned);
        return client.send(response).await;
    }

    // If the account hasn't accepted tos send the accept tos prompt
    if account.tos == false {
        let response = login_failed(LoginError::PromptTOS);
        client.session.account_id = account.id;
        return client.send(response).await;
    }

    packet.skip(6);
    let _hwid = packet.read_bytes(4);
    // TODO do stuff with hwid?

    client.session.account_id = account.id;
    client.session.pin = account.pin.clone();
    client.session.pic = account.pic.clone();
    client.session.tos = account.tos;
    client.update_state(LoginState::LoggedIn).await?;

    let response = login_succeeded(&account, config);
    client.send(response).await?;
    Ok(())
}

// TODO make core?
/// Error returned to client if login fails
enum LoginError {
    AccountBanned = 3,
    IncorrectPassword = 4,
    AccountNotFound = 5,
    TooManyAttempts = 6,
    AlreadyLoggedIn = 7,
    PromptTOS = 23,
}

/// Packet indicating login failed due to the given error
fn login_failed(error: LoginError) -> Packet {
    let mut packet = Packet::new();
    packet.write_short(0x00);
    packet.write_int(error as i32);
    packet.write_short(0);
    packet
}

/// Packet indicating login succeeded
pub fn login_succeeded(account: &account::Data, config: &Config) -> Packet {
    let mut packet = Packet::new();
    packet.write_short(0x00);
    packet.write_int(0);
    packet.write_short(0);
    packet.write_int(account.id);
    packet.write_byte(account.gender as u8);
    packet.write_byte(0);
    packet.write_byte(0);
    packet.write_byte(0);
    packet.write_string(&account.name);
    packet.write_byte(0);
    packet.write_byte(0);
    packet.write_long(0);
    packet.write_long(0);
    packet.write_int(1); // "select the world"... TODO test what this does
    packet.write_byte(config.enable_pin); // 0: enabled, 1: disabled
    packet.write_byte(config.enable_pic); // 0: register pic, 1: ask for pic, 2: disabled
    packet
}
