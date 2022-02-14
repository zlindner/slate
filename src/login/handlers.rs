use crate::{client::Client, login::packets, packet::Packet};

use deadpool_postgres::Pool;
use pbkdf2::{
    password_hash::{PasswordHash, PasswordVerifier},
    Pbkdf2,
};

// TODO clean up this struct
pub struct Account {
    pub id: i32,
    pub name: String,
    password_hash: String,
    banned: bool,
    accepted_tos: bool,
}

#[derive(Debug)]
pub enum LoginError {
    AccountNotFound = 5,
    InvalidPassword = 0,
    AccountBanned = 3,
    AcceptTOS = 23,
}

// TODO clean up accounts table
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

    // TODO check number of login attemps => not sure where to keep track

    let account = match get_account(name, &client.pool).await {
        Some(account) => account,
        None => {
            client
                .send_packet(packets::login_failed(LoginError::AccountNotFound))
                .await
                .unwrap();
            return;
        }
    };

    if let Err(e) = validate_account(&account, password).await {
        client.send_packet(packets::login_failed(e)).await.unwrap();
        return;
    }

    client
        .send_packet(packets::login_success(&account))
        .await
        .unwrap();
}

async fn get_account(name: String, pool: &Pool) -> Option<Account> {
    let client = pool.get().await.unwrap();
    let rows = client
        .query(
            "SELECT id, password, banned, tos FROM accounts WHERE name = $1",
            &[&name],
        )
        .await
        .unwrap();

    if rows.len() == 0 {
        return None;
    }

    let account = Account {
        id: rows[0].get(0),
        name: name,
        password_hash: rows[0].get(1),
        banned: rows[0].get(2),
        accepted_tos: rows[0].get(3),
    };

    Some(account)
}

async fn validate_account(account: &Account, password: String) -> Result<(), LoginError> {
    if account.banned {
        return Err(LoginError::AccountBanned);
    }

    // TODO check login state

    if !account.accepted_tos {
        // sends the accept tos modal
        return Err(LoginError::AcceptTOS);
    }

    // validate the entered password
    if let Err(e) = validate_password(account, password).await {
        return Err(e);
    }

    Ok(())
}

async fn validate_password(account: &Account, password: String) -> Result<(), LoginError> {
    // get the entered password's bytes
    let password = password.as_bytes();

    // get the account's hashed password
    let hash: &String = &account.password_hash;
    let parsed_hash = PasswordHash::new(hash).unwrap();

    // check the entered password against the parsed hash
    if Pbkdf2.verify_password(password, &parsed_hash).is_err() {
        return Err(LoginError::InvalidPassword);
    }

    Ok(())
}
