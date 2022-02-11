use deadpool_postgres::Pool;
use pbkdf2::{
    password_hash::{PasswordHash, PasswordVerifier},
    Pbkdf2,
};

use crate::packet::Packet;

#[derive(Debug)]
enum LoginError {
    AccountNotFound,
    InvalidPassword,
}

pub async fn login(mut packet: Packet, pool: &Pool) {
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

    // check if the account exists and validate the password
    if let Err(e) = validate_password(name, password, pool).await {
        log::error!("An error occurred while logging in: {:?}", e);
        return;
    }

    // TODO check if banned
    // TODO check if already logged in
    // TODO check for tos
}

async fn validate_password(name: String, password: String, pool: &Pool) -> Result<(), LoginError> {
    let client = pool.get().await.unwrap();
    let rows = client
        .query("SELECT password FROM accounts WHERE name = $1", &[&name])
        .await
        .unwrap();

    if rows.len() == 0 {
        log::debug!("Account not found with name: {}", name);
        return Err(LoginError::AccountNotFound);
    }

    // get and parse the hash stored in db
    let hash: String = rows[0].get(0);
    let parsed_hash = PasswordHash::new(&hash).unwrap();

    // check the entered password against the parsed hash
    let valid = Pbkdf2
        .verify_password(password.as_bytes(), &parsed_hash)
        .is_ok();

    if !valid {
        log::debug!("Invalid password for account with name: {}", name);
        return Err(LoginError::InvalidPassword);
    }

    Ok(())
}
