use oxide_core::net::{Connection, Packet};
use oxide_core::{Db, Redis, Result};

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

    pub async fn handle(self, connection: &mut Connection, db: &Db, redis: &Redis) -> Result<()> {
        use LoginServerPacketHandler::*;

        match self {
            Unknown(handler) => handler.handle(),
        }
    }
}
