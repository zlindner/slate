use crate::{
    client::Client,
    login::{packets, queries},
    net::packet::Packet,
    Result,
};

use bytes::Bytes;
use pbkdf2::{
    password_hash::{PasswordHash, PasswordVerifier},
    Pbkdf2,
};
use sqlx::{postgres::PgRow, Row};

#[derive(Debug)]
pub struct Login {
    name: String,
    password: String,
    hwid: Bytes,
}

#[derive(Debug)]
enum LoginError {
    InvalidPassword = 0,
    Banned = 3,
    NotFound = 5,
    TooManyAttempts = 6,
    InUse = 7,
    AcceptTOS = 23,
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

    pub async fn handle(self, client: &mut Client) -> Result<()> {
        let db = &client.db;
        let connection = &mut client.connection;

        client.login_attempts += 1;

        if client.login_attempts >= 5 {
            let packet = packets::login_failed(LoginError::TooManyAttempts as i32);
            connection.write_packet(packet).await?;
            client.disconnect().await?;
            return Ok(());
        }

        let account = match queries::get_account(&self.name, db).await {
            Ok(account) => account,
            Err(_) => {
                let packet = packets::login_failed(LoginError::NotFound as i32);
                connection.write_packet(packet).await?;
                return Ok(());
            }
        };

        if let Some(e) = self.validate_account(&account).await {
            let packet = packets::login_failed(e as i32);
            connection.write_packet(packet).await?;
        } else {
            let id = account.get::<i32, _>("id");
            let pin = account.get::<String, _>("pin");

            client.id = Some(id);

            if !pin.is_empty() {
                client.pin = Some(pin);
            }

            queries::update_login_state(id, 2, db).await?;

            let packet = packets::login_success(id, &self.name);
            connection.write_packet(packet).await?;
        }

        Ok(())
    }

    async fn validate_account(&self, account: &PgRow) -> Option<LoginError> {
        if account.get::<bool, _>("banned") {
            return Some(LoginError::Banned);
        }

        // 0 => logged out, 1 => transitioning, 2 => logged in
        if account.get::<i16, _>("login_state") != 0 {
            return Some(LoginError::InUse);
        }

        if !account.get::<bool, _>("accepted_tos") {
            return Some(LoginError::AcceptTOS);
        }

        // password entered in the client
        let password = self.password.as_bytes();
        // parse the hash stored in db
        let hash: String = account.get("password");
        let hash = PasswordHash::new(&hash).unwrap();

        // check the entered password against the hash
        if Pbkdf2.verify_password(password, &hash).is_err() {
            return Some(LoginError::InvalidPassword);
        }

        None
    }
}
