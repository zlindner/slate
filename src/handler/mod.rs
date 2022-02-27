use crate::client::Client;
use crate::net::packet::Packet;
use crate::Result;

mod login;
use self::login::Login;

mod world_status;
use self::world_status::WorldStatus;

mod world_list;
use self::world_list::WorldList;

mod unknown;
use self::unknown::Unknown;

pub enum Handler {
    Login(Login),
    WorldStatus(WorldStatus),
    WorldList(WorldList),
    Unknown(Unknown),
}

impl Handler {
    pub fn get(mut packet: Packet) -> Option<Self> {
        let op_code = packet.read_short();

        let handler = match op_code {
            0x01 => Handler::Login(Login::new(packet)),
            0x06 => Handler::WorldStatus(WorldStatus::new(packet)),
            0x0B => Handler::WorldList(WorldList::new()),
            _ => {
                if op_code >= 0x200 {
                    log::warn!("Potential malicious packet: {}", op_code);
                    return None;
                }

                Handler::Unknown(Unknown::new(op_code))
            }
        };

        Some(handler)
    }

    pub async fn handle(self, client: &mut Client) -> Result<()> {
        use Handler::*;

        match self {
            Login(handler) => handler.handle(client).await,
            WorldStatus(handler) => handler.handle(client).await,
            WorldList(handler) => handler.handle(client).await,
            Unknown(handler) => handler.handle(),
        }
    }
}
