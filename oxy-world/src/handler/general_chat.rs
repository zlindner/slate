use crate::{client::WorldClient, Shared, character::Character};
use anyhow::Result;
use oxy_core::net::Packet;

/// World server: general chat packet (0x31)
/// Called when a character sends a general chat message
pub async fn handle(mut packet: Packet, client: &mut WorldClient, shared: &Shared) -> Result<()> {
    let message = packet.read_string();

    // TODO can do some spam detection here

    // TODO allow gms to bypass this check?
    // Client tried to packet edit in general chat
    if message.len() > 127 {
        log::warn!(
            "Client tried to send general chat with length {}",
            message.len()
        );
        client.disconnect().await;
        return Ok(());
    }

    // TODO check if the message is a command
    // TODO check if map is muted

    let show = packet.read_byte();
    let map = shared.get_map(client.session.map_id);
    let character = map.characters.get(&client.session.character_id).unwrap();

    let broadcast = general_chat(&character, message, show);
    map.broadcast(broadcast, &character, true)?;

    Ok(())
}

///
fn general_chat(sender: &Character, message: String, show: u8) -> Packet {
    let mut packet = Packet::new();
    packet.write_short(0xA2);
    packet.write_int(sender.id);
    packet.write_byte(sender.data.gm as u8);
    packet.write_string(&message);
    packet.write_byte(show);
    packet
}
