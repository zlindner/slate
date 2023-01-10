use crate::{handler, Shared};
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
    pub map_id: i32,
    pub character_id: i32,
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
            map_id: -1,
            character_id: -1,
        }
    }

    pub async fn process(mut self, shared: Arc<Shared>) {
        if let Err(e) = self.on_connect().await {
            log::error!("Client connection error: {}", e);
            self.on_disconnect().await;
            return;
        }

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

                    if let Err(e) = handler::handle(packet, &mut self, &shared).await {
                        log::error!("Error handling packet: {}", e);
                    }
                }
                broadcast = self.broadcast_rx.recv() => {
                    let broadcast = match broadcast {
                        Ok(broadcast) => broadcast,
                        Err(e) => {
                            log::error!("Error receiving broadcast packet: {}", e);
                            break; // TODO does this break the loop or select?
                        }
                    };

                    // Check if we should send to the sender of the broadcast
                    if !broadcast.send_to_sender && broadcast.sender_character_id == self.session.character_id {
                        continue;
                    }

                    // TODO validate map id and position
                    if let Err(e) = self.stream.write_packet(broadcast.packet).await {
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
    pub async fn broadcast(&mut self, packet: Packet, send_to_sender: bool) -> Result<()> {
        // FIXME
        let broadcast_packet = BroadcastPacket {
            packet,
            sender_character_id: self.session.character_id,
            sender_map_id: 0,
            sender_position: (0, 0),
            send_to_sender,
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

        // TODO remove character from map
    }
}
