use super::{MapleCodec, Packet};
use anyhow::Result;
use futures::{SinkExt, StreamExt};
use tokio::net::TcpStream;
use tokio_util::codec::Framed;

/// A tokio TCP stream wrapped with custom maple encryption.
pub struct MapleStream {
    framed: Framed<TcpStream, MapleCodec>,
}

impl MapleStream {
    /// Creates a new MapleStream from the given TcpStream.
    pub fn new(stream: TcpStream) -> Self {
        Self {
            framed: Framed::new(stream, MapleCodec::new()),
        }
    }

    /// Reads and decrypts a packet from the TcpStream.
    pub async fn read_packet(&mut self) -> Option<Result<Packet>> {
        self.framed.next().await
    }

    /// Writes an encrypted packet to the TcpStream.
    pub async fn write_packet(&mut self, packet: Packet) -> Result<()> {
        self.framed.send(packet).await?;
        Ok(())
    }

    /// Writes a handshake packet to the TcpStream to setup encryption.
    pub async fn write_handshake(&mut self) -> Result<()> {
        let mut handshake = self.framed.codec().aes.get_handshake();
        handshake.use_encryption = false;
        self.write_packet(handshake).await?;
        Ok(())
    }

    /// Closes the underlying TcpStream.
    pub async fn close(&mut self) -> Result<()> {
        // TODO
        Ok(())
    }
}
