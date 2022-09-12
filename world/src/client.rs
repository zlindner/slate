use futures::SinkExt;
use oxide_core::{
    maple::Character,
    net::{codec::MapleCodec, Packet},
    Result,
};
use tokio::net::TcpStream;
use tokio_stream::StreamExt;
use tokio_util::codec::Framed;

pub struct Client {
    pub session_id: i32,
    pub stream: Framed<TcpStream, MapleCodec>,
    pub character: Option<Character>,
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
}
