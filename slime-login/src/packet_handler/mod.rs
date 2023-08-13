use crate::server::LoginSession;
use anyhow::Result;
use slime_net::Packet;

mod login;

/// Gets a packet handler for the given op code
pub async fn handle_packet(mut packet: Packet, session: &mut LoginSession) -> Result<()> {
    let op_code = packet.read_short();

    match op_code {
        0x01 => login::handle(packet, session).await?,
        /*0x05 => character_list::handle(packet, client, shared).await?,
        0x06 => world_status::handle(packet, client, shared).await?,
        0x07 => tos::handle(packet, client, shared).await?,
        0x09 => pin_operation::handle(packet, client).await?,
        0x0A => register_pin::handle(packet, client).await?,
        0x0B | 0x04 => world_list::handle(packet, client, shared).await?,
        0x13 => select_character::handle(packet, client).await?,
        0x15 => validate_character_name::handle(packet, client).await?,
        0x16 => create_character::handle(packet, client).await?,
        0x17 => delete_character::handle(packet, client, shared).await?,
        0x1D => register_pic::handle(packet, client).await?,
        0x1E => select_character_pic::handle(packet, client).await?,*/
        _ => log::debug!("Unhandled packet: {:02X?}", op_code),
    };

    Ok(())
}
