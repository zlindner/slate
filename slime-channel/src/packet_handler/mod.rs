use crate::session::ChannelSession;
use slime_net::Packet;

/// Gets a packet handler for the given op code
pub async fn handle_packet(mut packet: Packet, session: &mut ChannelSession) -> anyhow::Result<()> {
    let op_code = packet.read_short();

    match op_code {
        _ => log::info!("Unhandled packet: [{:02X?}]", op_code),
    };

    Ok(())
}
