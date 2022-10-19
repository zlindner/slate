use anyhow::Result;
use oxy_core::{
    net::{Client, Packet},
    prisma::account,
    prisma::LoginState,
};
use pbkdf2::{
    password_hash::{PasswordHash, PasswordVerifier},
    Pbkdf2,
};

/// Login server: login packet (0x01)
/// Called when the client clicks login after entering name and password
pub async fn handle(mut packet: Packet, client: &mut Client) -> Result<()> {
    if client.session.login_attempts >= 5 {
        let response = login_failed(LoginError::TooManyAttempts);
        client.send(response).await?;
        return Ok(());
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
            client.send(response).await?;
            return Ok(());
        }
    };

    if !matches!(account.state, LoginState::LoggedOut) {
        let response = login_failed(LoginError::AlreadyLoggedIn);
        client.send(response).await?;
        return Ok(());
    }

    if account.banned == true {
        let response = login_failed(LoginError::AccountBanned);
        client.send(response).await?;
        return Ok(());
    }

    // If the account hasn't accepted tos send the accept tos prompt
    if account.tos == false {
        let response = login_failed(LoginError::PromptTOS);
        client.send(response).await?;
        return Ok(());
    }

    let password = packet.read_string();
    let bytes = password.as_bytes();
    let db_hash = PasswordHash::new(&account.password).unwrap();

    // Validate the bytes of the entered password against the hash stored in db
    if Pbkdf2.verify_password(bytes, &db_hash).is_err() {
        let response = login_failed(LoginError::IncorrectPassword);
        client.send(response).await?;
        return Ok(());
    }

    packet.skip(6);
    let hwid = packet.read_bytes(4);
    // TODO do stuff with hwid?

    client.session.account_id = account.id;
    client.session.pin = account.pin.clone();
    client.session.pic = account.pic.clone();
    client.update_state(LoginState::LoggedIn).await?;

    let response = login_succeeded(&account);
    client.send(response).await?;
    Ok(())
}

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
    packet.write_byte(error as u8);
    packet.write_byte(0);
    packet.write_int(0);
    packet
}

/// Packet indicating login succeeded
fn login_succeeded(account: &account::Data) -> Packet {
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
    packet.write_byte(1); // 0: enable pin, 1: disable pin TODO config
    packet.write_byte(2); // 0: register pic, 1: ask for pic, 2: disabled TODO config
    packet
}
