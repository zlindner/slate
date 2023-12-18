use crate::{packet_handler, shutdown::Shutdown};
use crossbeam_channel::Receiver;
use dashmap::DashMap;
use slime_data::maple;
use slime_net::MapleStream;
use sqlx::{MySql, Pool};
use std::sync::Arc;
use tokio::sync::mpsc;

pub struct ChannelSession {
    pub id: i32,
    pub stream: MapleStream,
    pub db: Pool<MySql>,

    // Graceful shutdown handlers
    pub shutdown: Shutdown,
    pub _shutdown_complete: mpsc::Sender<()>,

    // TODO move to State struct
    pub maps: Arc<DashMap<i32, maple::Map>>,

    // Broadcast receiver for the current map
    pub broadcast_rx: Option<Receiver<maple::map::Broadcast>>,
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
                res = self.stream.read_packet() => {
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
                broadcast = async { self.broadcast_rx.as_mut().unwrap().recv() }, if self.broadcast_rx.is_some() => {
                    let broadcast = match broadcast {
                        Ok(broadcast) => broadcast,
                        Err(e) => {
                            log::error!("Error receiving broadcast packet: {} [id: {}]", e, self.id);
                            break;
                        }
                    };

                    /*if !broadcast.send_to_sender && broadcast.sender_id == self.session.character_id {
                        continue;
                    }*/

                    // TODO we can do some position check to only receive if we are in range?
                    // -> how to get our position?

                    if let Err(e) = self.stream.write_packet(broadcast.packet).await {
                        log::error!("Error writing broadcast packet: {} [id: {}]", e, self.id);
                    }
                }
                _ = self.shutdown.recv() => break,
            };
        }

        log::info!("Channel session ended [id: {}]", self.id);
    }
}
