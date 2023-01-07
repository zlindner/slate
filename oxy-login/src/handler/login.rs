use super::Config;
use crate::client::LoginClient;
use anyhow::Result;
use oxy_core::{net::Packet, prisma::account, prisma::LoginState};

/// Login server: login packet (0x01)
/// Called when the client clicks login after entering name and password
pub async fn handle(mut packet: Packet, client: &mut LoginClient, config: &Config) -> Result<()> {
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
    client.update_login_state(LoginState::LoggedIn).await?;

    let response = login_succeeded(&account, client, config);
    client.send(response).await?;
    Ok(())
}

/// Error returned to client if login fails
enum LoginError {
    AccountBanned = 0x03,
    IncorrectPassword = 0x04,
    AccountNotFound = 0x05,
    TooManyAttempts = 0x06,
    AlreadyLoggedIn = 0x07,
    PromptTOS = 0x17,
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
pub fn login_succeeded(account: &account::Data, client: &LoginClient, config: &Config) -> Packet {
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
    packet.write_byte(!config.enable_pin as u8); // 0: enabled, 1: disabled

    // 0: register pic (user hasn't registered pic)
    // 1: prompt pic (user already registered pic)
    // 2: disable pic
    let pic_flag = match config.enable_pic {
        true => !client.session.pic.is_empty() as u8,
        false => 2,
    };

    packet.write_byte(pic_flag);
    packet
}
