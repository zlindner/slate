use crate::handler::WorldPacketHandler;
use anyhow::{anyhow, Result};
use oxy_core::{
    net::{BroadcastPacket, MapleStream, Packet},
    prisma::{account, session, LoginState, PrismaClient},
};
use prisma_client_rust::chrono::{DateTime, FixedOffset, Utc};
use std::sync::Arc;
use tokio::{
    net::TcpStream,
    sync::broadcast::{Receiver, Sender},
};

pub struct WorldClient {
    stream: MapleStream,
    pub db: Arc<PrismaClient>,
    pub session: session::Data,
    broadcast_tx: Sender<BroadcastPacket>,
    broadcast_rx: Receiver<BroadcastPacket>,
    // TODO move character specific stuff to Character struct?
    pub position: (i32, i32),
    pub stance: i32,
}

impl WorldClient {
    pub fn new(
        stream: TcpStream,
        db: Arc<PrismaClient>,
        session_id: i32,
        broadcast_tx: Sender<BroadcastPacket>,
        broadcast_rx: Receiver<BroadcastPacket>,
    ) -> Self {
        let session = session::Data {
            id: session_id,
            account_id: -1,
            character_id: -1,
            world_id: -1,
            channel_id: -1,
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
            broadcast_tx,
            broadcast_rx,
            position: (0, 0),
            stance: 0,
        }
    }

    pub async fn process(mut self) {
        if let Err(e) = self.on_connect().await {
            log::error!("Client connection error: {}", e);
            self.on_disconnect().await;
            return;
        }

        let handler = WorldPacketHandler::new();

        loop {
            tokio::select! {
                // FIXME read_exact is not cancellation safe, not sure how to fix...
                // Can try to use tokio codec framed reads again, somehow remove select, ...
                packet = self.stream.read_packet() => {
                    let packet = match packet {
                        Ok(packet) => packet,
                        Err(e) => {
                            log::error!("Error reading broadcast packet: {}", e);
                            break; // TODO does this break the loop or select?
                        }
                    };

                    if let Err(e) = handler.handle(packet, &mut self).await {
                        log::error!("Error handling packet: {}", e);
                    }
                }
                broadcast_packet = self.broadcast_rx.recv() => {
                    let broadcast_packet = match broadcast_packet {
                        Ok(broadcast_packet) => broadcast_packet,
                        Err(e) => {
                            log::error!("Error receiving broadcast packet: {}", e);
                            break; // TODO does this break the loop or select?
                        }
                    };

                    if broadcast_packet.sender_character_id == self.session.character_id {
                        continue;
                    }

                    // TODO validate map id and position
                    if let Err(e) = self.stream.write_packet(broadcast_packet.packet).await {
                        log::error!("Error writing broadcast packet: {}", e);
                    }
                }
            };
        }

        self.on_disconnect().await;
    }

    /// Sends a packet to the client.
    pub async fn send(&mut self, packet: Packet) -> Result<()> {
        log::debug!("Sent: {}", packet);
        self.stream.write_packet(packet).await?;
        Ok(())
    }

    /// Broadcasts a packet to all other connected clients via a channel.
    /// Currently broadcasts to all connected clients (very inefficient).
    /// In future should implement broadcasting to specific world/channel/map.
    pub async fn broadcast(&mut self, packet: Packet) -> Result<()> {
        // FIXME
        let broadcast_packet = BroadcastPacket {
            packet,
            sender_character_id: self.session.character_id,
            sender_map_id: 0,
            sender_position: (0, 0),
        };

        self.broadcast_tx.send(broadcast_packet)?;
        Ok(())
    }

    /// Update the clients login state and last login time.
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

    /// Called when the client connects to the server.
    async fn on_connect(&mut self) -> Result<()> {
        log::info!("Client connected to server (session {})", self.session.id);
        self.stream.write_handshake().await?;
        Ok(())
    }

    /// Called when the client disconnects from the server.
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
