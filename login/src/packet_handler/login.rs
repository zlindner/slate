use crate::{client::Client, packets, queries};
use bytes::Bytes;
use oxide_core::{net::Packet, Db, Result};
use pbkdf2::{
    password_hash::{PasswordHash, PasswordVerifier},
    Pbkdf2,
};
use sqlx::FromRow;

enum LoginError {
    Banned = 3,
    InvalidPassword = 4,
    NotFound = 5,
    TooManyAttempts = 6,
    AlreadyLoggedIn = 7,
    AcceptTOS = 23,
}

#[derive(FromRow)]
struct Account {
    id: i32,
    name: String,
    password: String,
    pin: String,
    pic: String,
    login_state: i16,
    banned: bool,
    accepted_tos: bool,
}

pub struct Login {
    name: String,
    password: String,
    hwid: Bytes,
}

impl Login {
    pub fn new(mut packet: Packet) -> Self {
        let name = packet.read_string();
        let password = packet.read_string();
        packet.skip(6);
        let hwid = packet.read_bytes(4);

        Self {
            name,
            password,
            hwid,
        }
    }

    pub async fn handle(self, client: &mut Client, db: Db) -> Result<()> {
        if client.session.login_attempts >= 5 {
            let packet = packets::login_failed(LoginError::TooManyAttempts as i32);
            client.send(packet).await?;
            client.disconnect().await?;
            return Ok(());
        }

        client.session.login_attempts += 1;

        let account = match get_account(&self.name, &db).await {
            Ok(account) => account,
            Err(_) => {
                let packet = packets::login_failed(LoginError::NotFound as i32);
                client.send(packet).await?;
                return Ok(());
            }
        };

        let error = match account {
            Account { login_state: 1, .. } | Account { login_state: 2, .. } => {
                Some(LoginError::AlreadyLoggedIn)
            }
            Account { banned: true, .. } => Some(LoginError::Banned),
            Account {
                accepted_tos: false,
                ..
            } => Some(LoginError::AcceptTOS),
            _ => {
                // parse the hash stored in db
                let hash = PasswordHash::new(&account.password).unwrap();
                // check the entered password against the hash
                match Pbkdf2.verify_password(self.password.as_bytes(), &hash) {
                    Ok(_) => None,
                    Err(_) => Some(LoginError::InvalidPassword),
                }
            }
        };

        if error.is_some() {
            let packet = packets::login_failed(error.unwrap() as i32);
            client.send(packet).await?;
        } else {
            client.session.account_id = account.id;
            client.session.pin = account.pin;
            client.session.pic = account.pic;

            queries::update_login_state(account.id, 2, &db).await?;

            let packet = packets::login_success(account.id, &self.name);
            client.send(packet).await?;
        }

        Ok(())
    }
}

async fn get_account(name: &String, db: &Db) -> Result<Account> {
    let account: Account = sqlx::query_as(
        "SELECT id, name, password, pin, pic, login_state, last_login, banned, accepted_tos \
        FROM accounts \
        WHERE name = $1",
    )
    .bind(name)
    .fetch_one(db)
    .await?;

    Ok(account)
}
