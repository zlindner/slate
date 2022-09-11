use super::{codec::MapleCodec, Packet};
use crate::Result;
use bytes::BufMut;
use futures::SinkExt;
use tokio::net::TcpStream;
use tokio_stream::StreamExt;
use tokio_util::codec::Framed;

pub struct Connection {
    pub stream: Framed<TcpStream, MapleCodec>,
}

impl Connection {
    /// handshake packet sent immediately after a client establishes a connection with the server
    /// sets up client <-> server encryption via the passed initialization vectors and maple version
    pub async fn handshake(&mut self) -> Result<()> {
        let codec = self.stream.codec();

        let mut handshake = Packet::new();
        handshake.write_short(0x0E);
        // maple version
        handshake.write_short(83);
        // maple patch version
        handshake.write_string("1");
        // initialization vector for receive cipher
        handshake.write_bytes(&codec.recv.iv);
        // initialization vector for send cipher
        handshake.write_bytes(&codec.send.iv);
        // locale
        handshake.write_byte(8);

        self.write_raw_packet(handshake).await
    }

    pub async fn read_packet(&mut self) -> Result<Option<Packet>> {
        loop {
            return Ok(self.stream.try_next().await?);
        }
    }

    pub async fn write_packet(&mut self, packet: Packet) -> Result<()> {
        self.stream.send(packet).await
    }

    /// Writes a raw (unencrypted) packet directly to the streams write buffer
    pub async fn write_raw_packet(&mut self, packet: Packet) -> Result<()> {
        self.stream.write_buffer_mut().put_slice(&packet.bytes);
        self.stream.flush().await
    }

    pub async fn close(&mut self) -> Result<()> {
        self.stream.close().await
    }
}
