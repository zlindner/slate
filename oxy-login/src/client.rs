use crate::{handler, Shared};
use anyhow::{anyhow, Result};
use oxy_core::{
    net::{MapleStream, Packet},
    prisma::{account, session, LoginState, PrismaClient},
};
use prisma_client_rust::chrono::{DateTime, FixedOffset, Utc};
use std::sync::Arc;
use tokio::net::TcpStream;

pub struct LoginClient {
    stream: MapleStream,
    pub db: Arc<PrismaClient>,
    pub session: session::Data,
}

impl LoginClient {
    pub fn new(stream: TcpStream, db: Arc<PrismaClient>, session_id: i32) -> Self {
        let session = session::Data {
            id: session_id,
            account_id: -1,
            character_id: -1,
            world_id: -1,
            channel_id: -1,
            map_id: -1,
            login_attempts: 0,
            pin: String::new(),
            pin_attempts: 0,
            pic: String::new(),
            pic_attempts: 0,
            tos: false,
        };

        Self {
            stream: MapleStream::new(stream),
            db,
            session,
        }
    }

    ///
    pub async fn process(mut self, shared: Arc<Shared>) {
        if let Err(e) = self.on_connect().await {
            log::error!("Client connection error: {}", e);
            self.on_disconnect().await;
            return;
        }

        loop {
            let packet = match self.stream.read_packet().await {
                Ok(packet) => packet,
                Err(e) => {
                    log::error!("Error reading packet: {}", e);
                    break;
                }
            };

            if let Err(e) = handler::handle(packet, &mut self, &shared).await {
                log::error!("Error handling packet: {}", e);
            }
        }

        self.on_disconnect().await;
    }

    /// Sends a packet to the client.
    pub async fn send(&mut self, packet: Packet) -> Result<()> {
        log::debug!("Sent: {}", packet);
        self.stream.write_packet(packet).await?;
        Ok(())
    }

    /// Force disconnects the client from the connected server
    pub async fn disconnect(&mut self) {
        if let Err(e) = self.stream.close().await {
            log::error!(
                "Error disconnecting client (session {}): {}",
                self.session.id,
                e
            );
        }
    }

    /// Update the clients login state and last login time
    pub async fn update_login_state(&self, new_state: LoginState) -> Result<()> {
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

    /// Called when the client connects to the server
    async fn on_connect(&mut self) -> Result<()> {
        log::info!("Client connected to server (session {})", self.session.id);
        self.stream.write_handshake().await?;
        Ok(())
    }

    /// Called when the client disconnects from the server
    async fn on_disconnect(&self) {
        log::info!(
            "Client disconnected from server (session {})",
            self.session.id
        );

        if let Err(e) = self.update_login_state(LoginState::LoggedOut).await {
            log::debug!("Error updating login state: {}", e);
        }
    }
}
