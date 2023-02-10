use crate::{client::WorldClient, Shared};
use anyhow::Result;
use oxy_core::net::Packet;

mod connect;
mod general_chat;
mod move_character;
mod quest_action;

pub async fn handle(mut packet: Packet, client: &mut WorldClient, shared: &Shared) -> Result<()> {
    let op = packet.read_short();

    match op {
        0x14 => connect::handle(packet, client, shared).await?,
        0x29 => move_character::handle(packet, client, shared).await?,
        0x31 => general_chat::handle(packet, client, shared).await?,
        0x6B => quest_action::handle(packet, client, shared).await?,
        _ => log::debug!("Unhandled packet: {:02X?}", op),
    }

    Ok(())
}
