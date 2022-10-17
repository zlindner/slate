use super::Packet;
use crate::crypt::MapleAES;
use anyhow::{anyhow, Result};
use bytes::BytesMut;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
};

pub struct Client {
    stream: TcpStream,
    aes: MapleAES,
}

impl Client {
    pub fn new(stream: TcpStream) -> Self {
        Self {
            stream,
            aes: MapleAES::new(83),
        }
    }

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

    pub async fn send(&mut self, packet: Packet) -> Result<()> {
        Ok(())
    }

    /// Sends the handshake packet to the client to setup encryption
    /// Note: this panics if the handshake fails and disconnects the client
    pub async fn send_handshake(&mut self) {
        let handshake = self.aes.get_handshake();

        if let Err(e) = self.stream.write_all(&handshake.bytes).await {
            log::error!("Error sending handshake: {}", e);
        }

        if let Err(e) = self.stream.flush().await {
            log::error!("Error flushing stream: {}", e);
        }
    }
}
