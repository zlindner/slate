use crate::{packet_handler, shutdown::Shutdown, state::State};
use slate_data::{
    maple::{self, map::MapBroadcast},
    sql::{self, account::LoginState},
};
use slate_net::MapleStream;
use sqlx::{MySql, Pool};
use std::sync::Arc;
use tokio::sync::{broadcast, mpsc};

pub struct ChannelSession {
    pub id: i32,
    pub stream: MapleStream,
    pub db: Pool<MySql>,

    pub world_id: i32,
    pub channel_id: i32,
    pub account_id: Option<i32>,
    pub character: Option<maple::Character>,

    // Graceful shutdown handlers
    pub shutdown: Shutdown,
    pub _shutdown_complete: mpsc::Sender<()>,

    // Shared state
    pub state: Arc<State>,

    // Broadcast sender + receiver for the current map
    pub map_broadcast_tx: Option<broadcast::Sender<MapBroadcast>>,
    pub map_broadcast_rx: Option<broadcast::Receiver<MapBroadcast>>,
}

impl ChannelSession {
    /// Handles the channel session
    pub async fn handle(mut self) {
        log::info!("Created channel session [id: {}]", self.id);

        // Send unencrypted handshake to client -- sets up encryption IVs
        if let Err(e) = self.stream.write_handshake().await {
            log::error!("Handshake error: {} [id: {}]", e, self.id);
            return;
        }

        // Keep reading packets from the client in a loop until they disconnect,
        // an error occurs, or the server is shutting down
        while !self.shutdown.is_shutdown() {
            tokio::select! {
                res = { self.stream.read_packet() } => {
                    let packet = match res {
                        Some(Ok(packet)) => packet,
                        Some(Err(e)) => {
                            log::error!("Error reading packet: {} [id: {}]", e, self.id);
                            break;
                        }
                        // Client disconnected/sent EOF or shutdown signal was receieved
                        None => break,
                    };

                    if let Err(e) = packet_handler::handle_packet(packet, &mut self).await {
                        log::error!("Error handling packet: {} [id: {}]", e, self.id);
                    }
                },
                map_broadcast = async {
                    self.map_broadcast_rx.as_mut().unwrap().recv().await
                }, if self.map_broadcast_rx.is_some() => {
                    let map_broadcast = match map_broadcast {
                        Ok(map_broadcast) => map_broadcast,
                        Err(e) => {
                            log::error!("Error receiving broadcast packet: {} [id: {}]", e, self.id);
                            break;
                        }
                    };

                    self.handle_broadcast(map_broadcast).await;
                }
                _ = self.shutdown.recv() => break,
            };
        }

        log::info!("Channel session ended [id: {}]", self.id);

        if let Err(e) = self.on_disconnect().await {
            log::error!(
                "Error executing disconnection tasks: {} [id: {}]",
                e,
                self.id
            );
        }
    }

    /// Handles a map broadcast
    async fn handle_broadcast(&mut self, map_broadcast: MapBroadcast) {
        match map_broadcast {
            MapBroadcast::Packet(broadcast) => {
                // Check if we are the sender/if we want to send to sender
                if !broadcast.send_to_sender
                    && broadcast.sender_id == self.character.as_ref().unwrap().data.id
                {
                    return;
                }

                // We can optionally do some checks to see if we are in range to receive the broadcast

                if let Err(e) = self.stream.write_packet(broadcast.packet).await {
                    log::error!("Error writing broadcast packet: {} [id: {}]", e, self.id);
                }
            }
            MapBroadcast::Joined(sender) => {
                if let Err(e) = sender.send(self.character.as_ref().unwrap().clone()).await {
                    log::error!(
                        "Error responding to joined broadcast: {} [id: {}]",
                        e,
                        self.id
                    );
                }
            }
        }
    }

    /// Execute disconnection tasks
    async fn on_disconnect(&self) -> anyhow::Result<()> {
        sqlx::query(
            "UPDATE channels SET connected_players = connected_players - 1 WHERE world_id = ? AND id = ?"
        )
        .bind(self.world_id)
        .bind(self.channel_id)
        .execute(&self.db)
        .await?;

        if self.account_id.is_some() {
            sql::Account::update_login_state(
                self.account_id.unwrap(),
                LoginState::LoggedOut,
                &self.db,
            )
            .await?;
        }

        Ok(())
    }
}
