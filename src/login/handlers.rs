use deadpool_postgres::Pool;
use futures::SinkExt;
use pbkdf2::{
    password_hash::{PasswordHash, PasswordVerifier},
    Pbkdf2,
};

use crate::{client::Client, login::packets, packet::Packet};

struct LoginAccount {
    id: i32,
    name: String,
    password: String,
    banned: bool,
}

#[derive(Debug)]
pub enum LoginError {
    AccountNotFound = 5,
    InvalidPassword = 0,
    AccountBanned = 3,
}

pub async fn login(mut packet: Packet, client: &mut Client) {
    let name = packet.read_maple_string();
    let password = packet.read_maple_string();
    packet.advance(6);
    let hwid = packet.read_bytes(4);

    log::debug!(
        "Attempting to login: [name: {}, password: {}, hwid: {:?}]",
        name,
        password,
        hwid
    );

    let account = match get_account(name, &client.pool).await {
        Ok(account) => account,
        Err(e) => {
            log::error!("An error occurred while logging in: {:?}", e);
            client.stream.send(packets::login_failed(e)).await.unwrap();
            client.stream.flush().await.unwrap();

            return;
        }
    };

    // validate the entered password
    if let Err(e) = validate_password(account, password).await {
        log::error!("An error occurred while logging in: {:?}", e);
        return;
    }

    // TODO check if banned
    // TODO check if already logged in
    // TODO check for tos
}

async fn get_account(name: String, pool: &Pool) -> Result<LoginAccount, LoginError> {
    let client = pool.get().await.unwrap();
    let rows = client
        .query(
            "SELECT id, password, banned FROM accounts WHERE name = $1",
            &[&name],
        )
        .await
        .unwrap();

    if rows.len() == 0 {
        return Err(LoginError::AccountNotFound);
    }

    let account = LoginAccount {
        id: rows[0].get(0),
        name: name,
        password: rows[0].get(1),
        banned: rows[0].get(2),
    };

    Ok(account)
}

async fn validate_password(account: LoginAccount, password: String) -> Result<(), LoginError> {
    // get the entered password's bytes
    let password = password.as_bytes();

    // get the account's hashed password
    let hash: String = account.password;
    let parsed_hash = PasswordHash::new(&hash).unwrap();

    // check the entered password against the parsed hash
    if Pbkdf2.verify_password(password, &parsed_hash).is_err() {
        return Err(LoginError::InvalidPassword);
    }

    Ok(())
}
