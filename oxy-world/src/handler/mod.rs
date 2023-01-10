use crate::{client::WorldClient, Shared};
use anyhow::Result;
use oxy_core::net::Packet;
use std::sync::Arc;

mod connect;
mod move_character;

pub async fn handle(
    mut packet: Packet,
    client: &mut WorldClient,
    shared: &Arc<Shared>,
) -> Result<()> {
    let op = packet.read_short();

    match op {
        0x14 => connect::handle(packet, client, &shared).await?,
        0x29 => move_character::handle(packet, client, &shared).await?,
        _ => log::debug!("Unhandled packet: {:02X?}", op),
    }

    Ok(())
}
