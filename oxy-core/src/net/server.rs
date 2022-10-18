use super::Packet;
use crate::{net::Client, prisma::PrismaClient};
use anyhow::Result;
use async_trait::async_trait;
use std::sync::Arc;
use tokio::net::TcpListener;

pub struct Server;

impl Server {
    pub async fn start(addr: &str, events: impl Events, db: Arc<PrismaClient>) -> Result<()> {
        let listener = TcpListener::bind(addr).await?;
        let events = Arc::new(events);
        events.on_start(addr).await;

        let mut session_id = -1;

        loop {
            let (stream, _) = listener.accept().await?;
            let events = events.clone();
            let db = db.clone();
            session_id += 1;

            tokio::spawn(async move {
                let mut client = Client::new(stream, session_id);
                events.on_connect(&client).await;

                if let Err(e) = client.send_handshake().await {
                    log::error!("Error writing handshake: {}", e);
                    events.on_disconnect(&client).await;
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

                    events.on_packet(packet, &mut client, &db).await;
                }

                events.on_disconnect(&client).await;
            });
        }
    }
}

#[async_trait]
pub trait Events: Send + Sync + 'static {
    async fn on_start(&self, addr: &str);
    async fn on_connect(&self, client: &Client);
    async fn on_packet(&self, packet: Packet, client: &mut Client, db: &Arc<PrismaClient>);
    async fn on_disconnect(&self, client: &Client);
}
