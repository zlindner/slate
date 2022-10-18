use super::Packet;
use crate::{net::Client, prisma::PrismaClient};
use anyhow::Result;
use async_trait::async_trait;
use std::sync::Arc;
use tokio::net::TcpListener;

pub struct Server;

impl Server {
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

            tokio::spawn(async move {
                let mut client = Client::new(stream, db, session_id);

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
            });
        }
    }
}

#[async_trait]
pub trait HandlePacket: Send + Sync {
    async fn handle(&self, mut packet: Packet, client: &mut Client) -> Result<()>;
}
