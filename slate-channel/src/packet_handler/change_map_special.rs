use crate::{script_engine::PortalScriptEngine, session::ChannelSession};
use slate_data::{maple, packet};
use slate_net::Packet;

/// Channel server: change map special packet (0x64)
pub async fn handle(mut packet: Packet, session: &mut ChannelSession) -> anyhow::Result<()> {
    packet.skip(1);
    let start_wp = packet.read_string();
    packet.skip(2);

    log::debug!("start_wp: {}", start_wp);

    // TODO cache on character?
    let map = maple::Map::load(session.character.as_ref().unwrap().data.map)?;
    let portal = map.get_portal_by_name(start_wp);

    // TODO also check if character is transitioning maps?
    if portal.is_none() {
        return session.stream.write_packet(packet::enable_actions()).await;
    }

    // TODO if character has a trade, cancel it

    let portal = portal.unwrap();

    if session
        .character
        .as_ref()
        .unwrap()
        .blocked_portals
        .contains(&portal.script)
    {
        return session.stream.write_packet(packet::enable_actions()).await;
    }

    if !portal.script.is_empty() {
        PortalScriptEngine::execute_script(&portal.script);
    }

    // TODO else if portal target map id is not NONE -- change map to target map id

    Ok(())
}
