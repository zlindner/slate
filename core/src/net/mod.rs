use crate::Result;
use async_trait::async_trait;

pub mod cipher;
pub mod codec;
pub mod shanda;

pub(crate) mod packet;

pub use self::packet::Packet;

pub(crate) mod server;
pub use self::server::Server;

pub(crate) mod connection;
pub use self::connection::Connection;

#[async_trait]
pub trait HandlePacket {
    async fn handle(&self, packet: Packet, connection: &Connection) -> Result<()>;
}
