use crate::server::LoginSession;
use slime_net::Packet;

/// Login server: select character with pic packet (0x1E)
/// TODO
pub async fn handle(mut packet: Packet, session: &mut LoginSession) -> anyhow::Result<()> {
    if session.data.pic_attempts >= 6 {
        return session.stream.close().await;
    }

    session.data.pic_attempts += 1;
    let pic = packet.read_string();

    if session.data.pic.is_empty() || session.data.pic != pic {
        return session.stream.write_packet(invalid_pic()).await;
    }

    let character_id = packet.read_int();
    let mac_addr = packet.read_string();
    let host_addr = packet.read_string();

    session.data.character_id = character_id;
    super::select_character::connect_to_channel_server(session).await?;
    Ok(())
}

/// Packet indicating an invalid pic was entered
pub fn invalid_pic() -> Packet {
    let mut packet = Packet::new(0x1C);
    packet.write_byte(0);
    packet
}
