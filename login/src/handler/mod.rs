use oxide_core::net::{Connection, Packet};
use oxide_core::{Db, Result};

mod unknown;
use self::unknown::Unknown;

pub enum LoginServerHandler {
    Unknown(Unknown),
}

impl LoginServerHandler {
    pub fn get(mut packet: Packet) -> Self {
        let op_code = packet.read_short();

        match op_code {
            _ => Self::Unknown(Unknown::new(op_code)),
        }
    }

    pub async fn handle(self) -> Result<()> {
        use LoginServerHandler::*;

        match self {
            Unknown(handler) => handler.handle(),
        }
    }
}
