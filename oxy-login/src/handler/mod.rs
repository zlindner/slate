use crate::{client::LoginClient, Shared};
use anyhow::Result;
use oxy_core::net::Packet;

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

///
pub async fn handle(mut packet: Packet, client: &mut LoginClient, shared: &Shared) -> Result<()> {
    let op = packet.read_short();

    match op {
        0x01 => login::handle(packet, client, shared).await?,
        0x05 => character_list::handle(packet, client, shared).await?,
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
        0x1E => select_character_pic::handle(packet, client).await?,
        _ => log::debug!("Unhandled packet: {:02X?}", op),
    }

    Ok(())
}
