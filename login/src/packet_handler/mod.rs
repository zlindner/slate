use oxide_core::net::{Connection, Packet};
use oxide_core::{Db, Redis, Result};

mod login;
use self::login::Login;

mod character_list;
use self::character_list::CharacterList;

mod world_status;
use self::world_status::WorldStatus;

mod after_login;
use self::after_login::AfterLogin;

mod register_pin;
use self::register_pin::RegisterPin;

mod world_list;
use self::world_list::WorldList;

mod character_name;
use self::character_name::CharacterName;

mod create_character;
use self::create_character::CreateCharacter;

mod delete_character;
use self::delete_character::DeleteCharacter;

mod unknown;
use self::unknown::Unknown;

pub enum LoginServerPacketHandler {
    Login(Login),
    CharacterList(CharacterList),
    WorldStatus(WorldStatus),
    AfterLogin(AfterLogin),
    RegisterPin(RegisterPin),
    WorldList(WorldList),
    CharacterName(CharacterName),
    CreateCharacter(CreateCharacter),
    DeleteCharacter(DeleteCharacter),
    Unknown(Unknown),
}

impl LoginServerPacketHandler {
    pub fn get(mut packet: Packet) -> Self {
        let op_code = packet.read_short();

        match op_code {
            0x01 => Self::Login(Login::new(packet)),
            0x04 => Self::WorldList(WorldList::new()),
            0x05 => Self::CharacterList(CharacterList::new(packet)),
            0x06 => Self::WorldStatus(WorldStatus::new(packet)),
            0x09 => Self::AfterLogin(AfterLogin::new(packet)),
            0x0A => Self::RegisterPin(RegisterPin::new(packet)),
            0x0B => Self::WorldList(WorldList::new()),
            0x15 => Self::CharacterName(CharacterName::new(packet)),
            0x16 => Self::CreateCharacter(CreateCharacter::new(packet)),
            0x17 => Self::DeleteCharacter(DeleteCharacter::new(packet)),
            _ => Self::Unknown(Unknown::new(op_code)),
        }
    }

    pub async fn handle(self, connection: &mut Connection, db: Db, redis: Redis) -> Result<()> {
        use LoginServerPacketHandler::*;

        match self {
            Login(handler) => handler.handle(connection, db, redis).await,
            CharacterList(handler) => handler.handle(connection, db, redis).await,
            WorldStatus(handler) => handler.handle(connection).await,
            AfterLogin(handler) => handler.handle(connection, redis).await,
            RegisterPin(handler) => handler.handle(connection, db, redis).await,
            WorldList(handler) => handler.handle(connection).await,
            CharacterName(handler) => handler.handle(connection, db).await,
            CreateCharacter(handler) => handler.handle(connection, db, redis).await,
            DeleteCharacter(handler) => handler.handle(connection, db, redis).await,
            Unknown(handler) => handler.handle(),
        }
    }
}
