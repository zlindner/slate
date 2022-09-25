use crate::client::Client;
use oxide_core::net::Packet;
use oxide_core::{Db, Result};

mod connect;
use self::connect::Connect;

mod move_character;
use self::move_character::MoveCharacter;

mod auto_assign;
use self::auto_assign::AutoAssign;

mod unknown;
use self::unknown::Unknown;

pub enum WorldServerPacketHandler {
    Connect(Connect),
    MoveCharacter(MoveCharacter),
    AutoAssign(AutoAssign),
    Unknown(Unknown),
}

impl WorldServerPacketHandler {
    pub fn get(mut packet: Packet) -> Self {
        let op_code = packet.read_short();

        match op_code {
            0x14 => Self::Connect(Connect::new(packet)),
            0x29 => Self::MoveCharacter(MoveCharacter::new(packet)),
            0x58 => Self::AutoAssign(AutoAssign::new(packet)),
            _ => Self::Unknown(Unknown::new(op_code)),
        }
    }

    pub async fn handle(self, client: &mut Client, db: Db) -> Result<()> {
        use WorldServerPacketHandler::*;

        match self {
            Connect(handler) => handler.handle(client, db).await,
            MoveCharacter(handler) => handler.handle(client, db).await,
            AutoAssign(handler) => handler.handle(client, db).await,
            Unknown(handler) => handler.handle(),
        }
    }
}
