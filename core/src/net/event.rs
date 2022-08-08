use super::{Connection, Packet};
use async_trait::async_trait;

#[async_trait]
pub trait Events: Send + Sync {
    async fn on_start(&self, addr: &str);
    async fn on_shutdown(&self);
    async fn on_connect(&self, connection: &mut Connection);
    async fn on_packet(&self, connection: &mut Connection, packet: Packet);
    async fn on_disconnect(&self, connection: &mut Connection);
}
