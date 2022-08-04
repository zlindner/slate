use super::{codec::MapleCodec, Packet};
use crate::Result;
use bytes::BufMut;
use futures::SinkExt;
use tokio::net::TcpStream;
use tokio_stream::StreamExt;
use tokio_util::codec::{Decoder, Framed};

pub struct Connection {
    stream: Framed<TcpStream, MapleCodec>,
    pub session_id: usize,
}

impl Connection {
    pub fn new(stream: TcpStream, session_id: usize) -> Self {
        Self {
            stream: MapleCodec::new().framed(stream),
            session_id,
        }
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

    pub fn codec(&self) -> &MapleCodec {
        self.stream.codec()
    }
}
