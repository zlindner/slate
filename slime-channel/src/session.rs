use crate::{packet_handler, shutdown::Shutdown};
use slime_net::{MapleStream, Packet};
use sqlx::{MySql, Pool};
use tokio::sync::mpsc;

pub struct ChannelSession {
    pub id: i32,
    pub stream: MapleStream,
    pub db: Pool<MySql>,
    pub shutdown: Shutdown,
    pub _shutdown_complete: mpsc::Sender<()>,
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
            let maybe_packet = tokio::select! {
                // TODO this should eventually be get_next_packet, which selects between read_packet
                // and reading from broadcast channel
                res = self.stream.read_packet() => res,
                _ = self.shutdown.recv() => None,
            };

            let packet = match maybe_packet {
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
        }

        log::info!("Channel session ended [id: {}]", self.id);
    }
}
