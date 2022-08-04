use oxide_core::net::{Connection, Packet};
use oxide_core::{Db, Redis, Result};

mod login;
use self::login::Login;

mod unknown;
use self::unknown::Unknown;

pub enum LoginServerPacketHandler {
    Login(Login),
    Unknown(Unknown),
}

impl LoginServerPacketHandler {
    pub fn get(mut packet: Packet) -> Self {
        let op_code = packet.read_short();

        match op_code {
            0x01 => Self::Login(Login::new(packet)),
            _ => Self::Unknown(Unknown::new(op_code)),
        }
    }

    pub async fn handle(self, connection: &mut Connection, db: &Db, redis: &Redis) -> Result<()> {
        use LoginServerPacketHandler::*;

        match self {
            Login(handler) => handler.handle(connection, db, redis).await,
            Unknown(handler) => handler.handle(),
        }
    }
}
