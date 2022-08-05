use crate::state::State;
use oxide_core::net::{Connection, Packet};
use oxide_core::{Db, Result};
use std::sync::Arc;

mod login;
use self::login::Login;

mod after_login;
use self::after_login::AfterLogin;

mod register_pin;
use self::register_pin::RegisterPin;

mod unknown;
use self::unknown::Unknown;

pub enum LoginServerPacketHandler {
    Login(Login),
    AfterLogin(AfterLogin),
    RegisterPin(RegisterPin),
    Unknown(Unknown),
}

impl LoginServerPacketHandler {
    pub fn get(mut packet: Packet) -> Self {
        let op_code = packet.read_short();

        match op_code {
            0x01 => Self::Login(Login::new(packet)),
            0x09 => Self::AfterLogin(AfterLogin::new(packet)),
            0x0A => Self::RegisterPin(RegisterPin::new(packet)),
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
            AfterLogin(handler) => handler.handle(connection, state).await,
            RegisterPin(handler) => handler.handle(connection, db, state).await,
            Unknown(handler) => handler.handle(),
        }
    }
}
