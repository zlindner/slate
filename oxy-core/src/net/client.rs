use super::Packet;
use crate::{
    crypt::MapleAES,
    prisma::{session, PrismaClient},
};
use anyhow::{anyhow, Result};
use bytes::BytesMut;
use std::sync::Arc;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
};

pub struct Client {
    stream: TcpStream,
    aes: MapleAES,
    pub db: Arc<PrismaClient>,
    pub session: session::Data,
}

impl Client {
    pub fn new(stream: TcpStream, db: Arc<PrismaClient>, session_id: i32) -> Self {
        let session = session::Data { id: session_id };

        Self {
            stream,
            db,
            aes: MapleAES::new(83),
            session,
        }
    }

    /// Reads a packet from the client
    pub async fn read(&mut self) -> Result<Packet> {
        let mut header = [0u8; 4];
        self.stream.read_exact(&mut header).await?;

        if !self.aes.is_valid_header(&header) {
            return Err(anyhow!("Invalid packet header: {:02X?}", header));
        }

        let len = self.aes.get_packet_len(&header);
        let mut body = BytesMut::zeroed(len as usize);
        self.stream.read_exact(&mut body).await?;
        self.aes.decrypt(&mut body);
        Ok(Packet::wrap(body))
    }

    /// Sends a packet to the client
    pub async fn send(&mut self, packet: Packet) -> Result<()> {
        Ok(())
    }

    /// Sends the handshake packet to the client to setup encryption
    pub async fn send_handshake(&mut self) -> Result<()> {
        let handshake = self.aes.get_handshake();
        self.stream.write_all(&handshake.bytes).await?;
        Ok(())
    }

    pub async fn on_connect(&mut self) -> Result<()> {
        log::info!("Client connected to server (session {})", self.session.id);
        self.send_handshake().await?;
        Ok(())
    }

    pub async fn on_disconnect(&self) {
        log::info!(
            "Client disconnected from server (session {})",
            self.session.id
        );

        // TODO update login state
    }
}
