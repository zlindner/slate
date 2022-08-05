use oxide_core::net::{Connection, Packet};
use oxide_core::{Db, Result};
use std::sync::Arc;

mod login;

use crate::state::State;

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

    pub async fn handle(
        self,
        connection: &mut Connection,
        db: &Db,
        state: Arc<State>,
    ) -> Result<()> {
        use LoginServerPacketHandler::*;

        match self {
            Login(handler) => handler.handle(connection, db, state).await,
            Unknown(handler) => handler.handle(),
        }
    }
}
