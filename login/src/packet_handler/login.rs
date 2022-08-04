use crate::packets;
use bytes::Bytes;
use deadpool_redis::redis::cmd;
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
    InvalidPassword = 0,
    Banned = 3,
    NotFound = 5,
    TooManyAttempts = 6,
    InUse = 7,
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

    // TODO look into using a hashmap for each session / user
    // can probably save us from a bunch of db calls?
    // https://redis.io/commands/?group=hash
    pub async fn handle(self, connection: &mut Connection, db: &Db, redis: &Redis) -> Result<()> {
        let mut r = redis.get().await?;

        // TODO need to handle case where key doesn't exist yet?
        let key = format!("login/login_attempts/{}", connection.session_id);
        let mut login_attempts: i32 = cmd("GET").arg(&key).query_async(&mut r).await?;

        if login_attempts >= 5 {
            let packet = packets::login_failed(LoginError::TooManyAttempts as i32);
            connection.write_packet(packet).await?;
            // TODO move to on_disconnect or something
            cmd("DEL").arg(key).query_async(&mut r).await?;
            // TODO disconnect client
            return Ok(());
        }

        login_attempts += 1;

        cmd("SET")
            .arg(&[key, login_attempts.to_string()])
            .query_async(&mut r)
            .await?;

        let account = match get_account(&self.name, db).await {
            Ok(account) => account,
            Err(err) => {
                log::error!("get_account error: {}", err);
                connection
                    .write_packet(packets::login_failed(LoginError::NotFound as i32))
                    .await?;
                return Ok(());
            }
        };

        let error = match account {
            Account { login_state: 1, .. } | Account { login_state: 2, .. } => {
                Some(LoginError::InUse)
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
            connection.write_packet(packet).await?;
        } else {
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
