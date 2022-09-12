use futures::SinkExt;
use oxide_core::{
    net::{codec::MapleCodec, Packet},
    pg::Session,
    Result,
};
use tokio::net::TcpStream;
use tokio_stream::StreamExt;
use tokio_util::codec::Framed;

pub struct Client {
    pub stream: Framed<TcpStream, MapleCodec>,
    pub session: Session,
}

impl Client {
    pub async fn read(&mut self) -> Result<Option<Packet>> {
        loop {
            return Ok(self.stream.try_next().await?);
        }
    }

    pub async fn send(&mut self, packet: Packet) -> Result<()> {
        self.stream.send(packet).await
    }

    pub async fn disconnect(&mut self) -> Result<()> {
        self.stream.close().await
    }
}
