use super::{cipher::Cipher, codec::MapleCodec, Packet};
use crate::{util::Shutdown, Result};
use futures::Future;
use tokio::net::TcpStream;
use tokio_stream::StreamExt;
use tokio_util::codec::{Decoder, Framed};

pub struct Connection {
    stream: Framed<TcpStream, MapleCodec>,
}

impl Connection {
    pub fn new(stream: TcpStream) -> Self {
        let send = Cipher::new(0xffff - 83);
        let recv = Cipher::new(83);

        Self {
            stream: MapleCodec::new(send, recv).framed(stream),
        }
    }

    pub async fn connect<Fut>(
        &mut self,
        mut shutdown: Shutdown,
        handler: impl FnOnce(Packet) -> Fut + Send + Copy + 'static,
    ) -> Result<()>
    where
        Fut: Future<Output = Result<()>>,
    {
        while !shutdown.is_shutdown() {
            let maybe_packet = tokio::select! {
                res = self.read_packet() => res?,
                _ = shutdown.recv() => {
                    return Ok(());
                }
            };

            // None => client disconnected
            let packet = match maybe_packet {
                Some(packet) => packet,
                None => return Ok(()),
            };

            log::debug!("Received packet: {}", packet);
            handler(packet).await?;
        }

        Ok(())
    }

    async fn read_packet(&mut self) -> Result<Option<Packet>> {
        loop {
            return Ok(self.stream.try_next().await?);
        }
    }

    pub async fn disconnect(&mut self) -> Result<()> {
        Ok(())
    }
}
