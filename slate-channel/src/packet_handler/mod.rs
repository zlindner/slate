use crate::session::ChannelSession;
use slate_net::Packet;

mod connect;
mod move_character;
mod quest_action;

/// Gets a packet handler for the given op code
pub async fn handle_packet(mut packet: Packet, session: &mut ChannelSession) -> anyhow::Result<()> {
    let op_code = packet.read_short();

    match op_code {
        0x14 => connect::handle(packet, session).await?,
        0x29 => move_character::handle(packet, session).await?,
        0x6B => quest_action::handle(packet, session).await?,
        _ => log::info!("Unhandled packet: [{:02X?}]", op_code),
    };

    Ok(())
}
