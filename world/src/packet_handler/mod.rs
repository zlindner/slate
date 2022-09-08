use oxide_core::net::{Connection, Packet};
use oxide_core::{Db, Redis, Result};

mod connect;
use self::connect::Connect;

mod unknown;
use self::unknown::Unknown;

pub enum WorldServerPacketHandler {
    Connect(Connect),
    Unknown(Unknown),
}

impl WorldServerPacketHandler {
    pub fn get(mut packet: Packet) -> Self {
        let op_code = packet.read_short();

        match op_code {
            0x14 => Self::Connect(Connect::new(packet)),
            _ => Self::Unknown(Unknown::new(op_code)),
        }
    }

    pub async fn handle(self, connection: &mut Connection, db: Db, redis: Redis) -> Result<()> {
        use WorldServerPacketHandler::*;

        match self {
            Connect(handler) => handler.handle(connection, db, redis).await,
            Unknown(handler) => handler.handle(),
        }
    }
}
