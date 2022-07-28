use async_trait::async_trait;
use oxide_core::net::{HandlePacket, Packet};
use oxide_core::Result;

mod unknown;
use self::unknown::Unknown;

#[derive(Copy, Clone)]
pub struct LoginServerHandler;

#[async_trait]
impl HandlePacket for LoginServerHandler {
    async fn handle(&self, packet: Packet) -> Result<()> {
        Handler::get(packet).handle().await
    }
}

enum Handler {
    Unknown(Unknown),
}

impl Handler {
    fn get(mut packet: Packet) -> Self {
        let op_code = packet.read_short();

        match op_code {
            _ => Self::Unknown(Unknown::new(op_code)),
        }
    }

    async fn handle(self) -> Result<()> {
        use Handler::*;

        match self {
            Unknown(handler) => handler.handle(),
        }
    }
}
