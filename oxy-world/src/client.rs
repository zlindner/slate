use crate::{handler, Shared};
use anyhow::Result;
use oxy_core::{
    net::{BroadcastPacket, MapleStream, Packet},
    prisma::{session, LoginState, PrismaClient},
    queries,
};
use std::sync::Arc;
use tokio::{net::TcpStream, sync::broadcast::Receiver};

pub struct WorldClient {
    stream: MapleStream,
    pub broadcast_rx: Option<Receiver<BroadcastPacket>>,
    pub session: session::Data,
}

impl WorldClient {
    pub fn new(stream: TcpStream, session_id: i32) -> Self {
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
            broadcast_rx: None,
            session,
        }
    }

    pub async fn process(mut self, shared: Arc<Shared>) {
        if let Err(e) = self.on_connect().await {
            log::error!("Client connection error: {}", e);
            self.on_disconnect(&shared.db).await;
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
                            log::error!("Error reading packet: {}", e);
                            break;
                        }
                    };

                    if let Err(e) = handler::handle(packet, &mut self, &shared).await {
                        log::error!("Error handling packet: {}", e);
                    }
                }
                broadcast = async { self.broadcast_rx.as_mut().unwrap().recv().await }, if self.broadcast_rx.is_some() => {
                    let broadcast = match broadcast {
                        Ok(broadcast) => broadcast,
                        Err(e) => {
                            log::error!("Error receiving broadcast packet: {}", e);
                            break;
                        }
                    };

                    // TODO we can do some position check to only receive if we are in range

                    if !broadcast.send_to_sender && broadcast.sender_id == self.session.character_id {
                        continue;
                    }

                    log::debug!("Sending broadcast...");
                    
                    if let Err(e) = self.send(broadcast.packet).await {
                        log::error!("Error writing broadcast packet: {}", e);
                    }
                }
            };
        }

        self.on_disconnect(&shared.db).await;
    }

    /// Sends a packet to the client.
    pub async fn send(&mut self, packet: Packet) -> Result<()> {
        log::debug!("Sent: {}", packet);
        self.stream.write_packet(packet).await?;
        Ok(())
    }

    /// Called when the client connects to the server.
    async fn on_connect(&mut self) -> Result<()> {
        log::info!("Client connected to server (session {})", self.session.id);
        self.stream.write_handshake().await?;
        Ok(())
    }

    /// Called when the client disconnects from the server.
    async fn on_disconnect(&self, db: &PrismaClient) {
        log::info!(
            "Client disconnected from server (session {})",
            self.session.id
        );

        if let Err(e) =
            queries::update_login_state(db, self.session.account_id, LoginState::LoggedOut).await
        {
            log::debug!("Error updating login state: {}", e);
        }

        // TODO remove character from map
    }
}
