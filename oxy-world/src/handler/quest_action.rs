use crate::{client::WorldClient, Shared};
use anyhow::Result;
use oxy_core::net::Packet;

/// World server: quest action packet (0x6B)
///
pub async fn handle(mut packet: Packet, client: &mut WorldClient, shared: &Shared) -> Result<()> {
    let action = packet.read_byte();
    let quest_id = packet.read_short();

    match action {
        // Restore lost item
        0 => {
            packet.skip(4);
            let item_id = packet.read_int();
            // TODO restore lost item
        }
        // Start quest
        1 => {
            let npc = packet.read_int();
            // TODO check if npc is nearby
            // TODO check if quest can start
            // TODO start quest
        }
        // Complete quest
        2 => {
            //
        }
        // Forfeit quest
        3 => {
            //
        }
        // Scripted start quest
        4 => {
            //
        }
        // Scripted end quest
        5 => {
            //
        }
        _ => {
            log::debug!("Invalid quest action: {}", action);
        }
    }

    Ok(())
}
