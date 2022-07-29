use super::{codec::MapleCodec, Packet};
use async_trait::async_trait;
use tokio::net::TcpStream;
use tokio_util::codec::Framed;

#[async_trait]
pub trait Events: Send + Sync {
    async fn on_start(&self, addr: &str);
    async fn on_connect(&self, stream: &mut Framed<TcpStream, MapleCodec>);
    async fn on_packet(&self, stream: &mut Framed<TcpStream, MapleCodec>, packet: Packet);
    async fn on_disconnect(&self, stream: &mut Framed<TcpStream, MapleCodec>);
}
