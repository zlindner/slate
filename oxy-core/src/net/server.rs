use super::HandlePacket;
use crate::{net::Client, prisma::PrismaClient};
use anyhow::Result;
use std::sync::Arc;
use tokio::net::TcpListener;

pub struct Server;

impl Server {
    /// Start listening at the given addr and process incoming connections
    pub async fn start(
        addr: &str,
        handler: &'static impl HandlePacket,
        db: Arc<PrismaClient>,
    ) -> Result<()> {
        let listener = TcpListener::bind(addr).await?;
        log::info!("Server started @ {}", addr);

        let mut session_id = 0;

        loop {
            let (stream, _) = listener.accept().await?;
            let db = db.clone();
            session_id += 1;

            // Create a new task per connection/client
            tokio::spawn(async move {
                let client = Client::new(stream, db, session_id);
                Self::process(client, handler).await;
            });
        }
    }

    /// Process the connection/client: send handshake and handle incoming packets
    async fn process(mut client: Client, handler: &'static impl HandlePacket) {
        if let Err(e) = client.on_connect().await {
            log::error!("Client connection error: {}", e);
            client.on_disconnect().await;
            return;
        }

        loop {
            let packet = match client.read().await {
                Ok(packet) => packet,
                Err(e) => {
                    log::error!("Error reading packet: {}", e);
                    break;
                }
            };

            if let Err(e) = handler.handle(packet, &mut client).await {
                log::error!("Error handling packet: {}", e);
            }
        }

        client.on_disconnect().await;
    }
}
