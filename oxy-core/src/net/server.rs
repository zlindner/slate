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

        loop {
            let (stream, _) = listener.accept().await?;
            let events = events.clone();
            let db = db.clone();

            tokio::spawn(async move {
                let mut client = Client::new(stream);
                client.send_handshake().await;

                loop {
                    let packet = match client.read().await {
                        Ok(packet) => packet,
                        Err(_) => break,
                    };

                    events.on_packet(packet, &db).await;
                }

                events.on_disconnect().await;
            });
        }
    }
}

#[async_trait]
pub trait Events: Send + Sync + 'static {
    async fn on_start(&self, addr: &str);
    async fn on_connect(&self);
    async fn on_packet(&self, packet: Packet, db: &Arc<PrismaClient>);
    async fn on_disconnect(&self);
}
