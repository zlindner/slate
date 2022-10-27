use super::Packet;
use crate::{
    crypt::MapleAES,
    prisma::{account, session, LoginState, PrismaClient},
};
use anyhow::{anyhow, Result};
use bytes::BytesMut;
use prisma_client_rust::chrono::{DateTime, FixedOffset, Utc};
use std::sync::Arc;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
};

pub struct Client {
    stream: TcpStream,
    aes: MapleAES,
    pub db: Arc<PrismaClient>,
    pub session: session::Data,
}

impl Client {
    pub fn new(stream: TcpStream, db: Arc<PrismaClient>, session_id: i32) -> Self {
        let session = session::Data {
            id: session_id,
            account_id: -1,
            character_id: -1,
            world_id: -1,
            channel_id: -1,
            login_attempts: 0,
            pin: "".to_string(),
            pin_attempts: 0,
            pic: "".to_string(),
            pic_attempts: 0,
            tos: false,
        };

        Self {
            stream,
            db,
            aes: MapleAES::new(83),
            session,
        }
    }

    /// Reads a packet from the client
    pub async fn read(&mut self) -> Result<Packet> {
        let mut header = [0u8; 4];
        self.stream.read_exact(&mut header).await?;

        if !self.aes.is_valid_header(&header) {
            return Err(anyhow!("Invalid packet header: {:02X?}", header));
        }

        let len = self.aes.get_packet_len(&header);
        let mut body = BytesMut::zeroed(len as usize);
        self.stream.read_exact(&mut body).await?;
        self.aes.decrypt(&mut body);
        Ok(Packet::wrap(body))
    }

    /// Sends a packet to the client
    pub async fn send(&mut self, mut packet: Packet) -> Result<()> {
        log::debug!("Sent: {}", packet);
        let header = self.aes.build_header(packet.len());
        self.aes.encrypt(&mut packet.bytes);
        self.stream.write_all(&header).await?;
        self.stream.write_all(&packet.bytes).await?;
        Ok(())
    }

    /// Sends the handshake packet to the client to setup encryption
    pub async fn send_handshake(&mut self) -> Result<()> {
        let handshake = self.aes.get_handshake();
        self.stream.write_all(&handshake.bytes).await?;
        Ok(())
    }

    /// Called when the client connects to the server
    pub async fn on_connect(&mut self) -> Result<()> {
        log::info!("Client connected to server (session {})", self.session.id);
        self.send_handshake().await?;
        Ok(())
    }

    /// Called when the client disconnects from the server
    pub async fn on_disconnect(&self) {
        log::info!(
            "Client disconnected from server (session {})",
            self.session.id
        );

        if let Err(e) = self.update_state(LoginState::LoggedOut).await {
            log::debug!("Error updating login state: {}", e);
        }
    }

    /// Update the clients login state and last login time
    pub async fn update_state(&self, new_state: LoginState) -> Result<()> {
        if self.session.account_id == -1 {
            return Err(anyhow!("Error updating state: invalid account id"));
        }

        let now: DateTime<FixedOffset> = DateTime::from(Utc::now());

        self.db
            .account()
            .update(
                account::id::equals(self.session.account_id),
                vec![
                    account::state::set(new_state),
                    account::last_login::set(Some(now)),
                ],
            )
            .exec()
            .await?;

        Ok(())
    }

    /// Get the client's account by account id stored in session
    pub async fn get_account(&self) -> Result<Option<account::Data>> {
        if self.session.account_id == -1 {
            return Err(anyhow!("Error getting account: invalid account id"));
        }

        let account = self
            .db
            .account()
            .find_unique(account::id::equals(self.session.account_id))
            .exec()
            .await?;

        Ok(account)
    }
}
