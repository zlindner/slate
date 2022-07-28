use super::{cipher::Cipher, codec::MapleCodec, HandlePacket};
use crate::{util::Shutdown, Result};
use tokio::net::TcpStream;
use tokio_stream::StreamExt;
use tokio_util::codec::{Decoder, Framed};

pub struct Connection {
    stream: Framed<TcpStream, MapleCodec>,
    shutdown: Shutdown,
}

impl Connection {
    pub fn new(stream: TcpStream, shutdown: Shutdown) -> Self {
        let send = Cipher::new(0xffff - 83);
        let recv = Cipher::new(83);

        Self {
            stream: MapleCodec::new(send, recv).framed(stream),
            shutdown: shutdown,
        }
    }

    pub async fn read_packets(&mut self, handler: &impl HandlePacket) -> Result<()> {
        while !self.shutdown.is_shutdown() {
            let maybe_packet = tokio::select! {
                res = self.stream.try_next() => res?,
                _ = self.shutdown.recv() => {
                    return Ok(());
                }
            };

            // None => client disconnected
            let packet = match maybe_packet {
                Some(packet) => packet,
                None => return Ok(()),
            };

            log::debug!("Received packet: {}", packet);
            HandlePacket::handle(handler, packet, self).await?;
        }

        Ok(())
    }

    pub async fn disconnect(&mut self) -> Result<()> {
        Ok(())
    }
}
