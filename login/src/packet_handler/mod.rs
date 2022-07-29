use oxide_core::net::codec::MapleCodec;
use oxide_core::net::Packet;
use oxide_core::Result;
use tokio::net::TcpStream;
use tokio_util::codec::Framed;

mod unknown;
use self::unknown::Unknown;

pub enum LoginServerPacketHandler {
    Unknown(Unknown),
}

impl LoginServerPacketHandler {
    pub fn get(mut packet: Packet) -> Self {
        let op_code = packet.read_short();

        match op_code {
            _ => Self::Unknown(Unknown::new(op_code)),
        }
    }

    pub async fn handle(self, stream: &mut Framed<TcpStream, MapleCodec>) -> Result<()> {
        use LoginServerPacketHandler::*;

        match self {
            Unknown(handler) => handler.handle(),
        }
    }
}
