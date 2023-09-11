use crate::server::LoginSession;
use slime_net::Packet;

mod character_list;
mod create_character;
mod delete_character;
mod login;
mod pin_operation;
mod register_pic;
mod register_pin;
mod select_character;
mod select_character_pic;
mod tos;
mod validate_character_name;
mod world_list;
mod world_status;

/// Gets a packet handler for the given op code
pub async fn handle_packet(mut packet: Packet, session: &mut LoginSession) -> anyhow::Result<()> {
    let op_code = packet.read_short();

    match op_code {
        0x01 => login::handle(packet, session).await?,
        0x05 => character_list::handle(packet, session).await?,
        0x06 => world_status::handle(packet, session).await?,
        0x07 => tos::handle(packet, session).await?,
        0x09 => pin_operation::handle(packet, session).await?,
        0x0A => register_pin::handle(packet, session).await?,
        0x0B | 0x04 => world_list::handle(packet, session).await?,
        0x13 => select_character::handle(packet, session).await?,
        0x15 => validate_character_name::handle(packet, session).await?,
        0x16 => create_character::handle(packet, session).await?,
        0x17 => delete_character::handle(packet, session).await?,
        0x1D => register_pic::handle(packet, session).await?,
        0x1E => select_character_pic::handle(packet, session).await?,
        _ => log::debug!("Unhandled packet: {:02X?}", op_code),
    };

    Ok(())
}
