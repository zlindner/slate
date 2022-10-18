use async_trait::async_trait;
use anyhow::Result;

pub(crate) mod client;
pub use self::client::Client;

pub(crate) mod packet;
pub use self::packet::Packet;

pub(crate) mod server;
pub use self::server::Server;

#[async_trait]
pub trait HandlePacket: Send + Sync {
    async fn handle(&self, mut packet: Packet, client: &mut Client) -> Result<()>;
}
