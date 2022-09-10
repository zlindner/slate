use crate::{packets, queries};
use bytes::Bytes;
use deadpool_redis::redis::AsyncCommands;
use oxide_core::{
    net::{Connection, Packet},
    Db, Redis, Result,
};
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

    pub async fn handle(self, connection: &mut Connection, db: Db, redis: Redis) -> Result<()> {
        let mut redis = redis.get().await?;
        let key = format!("login_session:{}", connection.session_id);
        let mut login_attempts: u8 = redis.hget(&key, "login_attempts").await?;
        login_attempts += 1;

        if login_attempts >= 5 {
            let packet = packets::login_failed(LoginError::TooManyAttempts as i32);
            connection.write_packet(packet).await?;
            connection.close().await?;
            return Ok(());
        }

        let account = match get_account(&self.name, &db).await {
            Ok(account) => account,
            Err(_) => {
                redis.hset(&key, "login_attempts", login_attempts).await?;
                connection
                    .write_packet(packets::login_failed(LoginError::NotFound as i32))
                    .await?;

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
            redis.hset(&key, "login_attempts", login_attempts).await?;
            connection.write_packet(packet).await?;
        } else {
            redis
                .hset_multiple(
                    key,
                    &[
                        ("account_id", account.id.to_string()),
                        ("pin", account.pin),
                        ("pic", account.pic),
                    ],
                )
                .await?;

            queries::update_login_state(account.id, 2, &db).await?;

            let packet = packets::login_success(account.id, &self.name);
            connection.write_packet(packet).await?;
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
