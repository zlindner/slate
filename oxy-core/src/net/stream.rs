use super::Packet;
use crate::crypt::MapleAES;
use anyhow::{anyhow, Result};
use bytes::BytesMut;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
};

/// A tokio TCP stream wrapped with custom maple encryption.
pub struct MapleStream {
    stream: TcpStream,
    aes: MapleAES,
}

impl MapleStream {
    /// Creates a new MapleStream from the given TcpStream.
    pub fn new(stream: TcpStream) -> Self {
        Self {
            stream,
            aes: MapleAES::new(83),
        }
    }

    /// Reads and decrypts a packet from the TcpStream.
    pub async fn read_packet(&mut self) -> Result<Packet> {
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

    /// Writes an encrypted packet to the TcpStream.
    pub async fn write_packet(&mut self, mut packet: Packet) -> Result<()> {
        let header = self.aes.build_header(packet.len());
        self.aes.encrypt(&mut packet.bytes);
        self.stream.write_all(&header).await?;
        self.stream.write_all(&packet.bytes).await?;
        Ok(())
    }

    /// Writes a handshake packet to the TcpStream to setup encryption.
    pub async fn write_handshake(&mut self) -> Result<()> {
        let handshake = self.aes.get_handshake();
        self.stream.write_all(&handshake.bytes).await?;
        Ok(())
    }

    /// Closes the underlying TcpStream.
    pub async fn close(&mut self) -> Result<()> {
        self.stream.shutdown().await?;
        Ok(())
    }
}
