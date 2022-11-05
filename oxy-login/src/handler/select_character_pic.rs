use anyhow::Result;
use oxy_core::net::{Client, Packet};

/// Login server: select character with pic packet (0x1E)
///
pub async fn handle(mut packet: Packet, client: &mut Client) -> Result<()> {
    if client.session.pic_attempts >= 6 {
        client.disconnect().await;
        return Ok(());
    }

    client.session.pic_attempts += 1;
    let pic = packet.read_string();

    if client.session.pic.is_empty() || client.session.pic != pic {
        let response = invalid_pic();
        return client.send(response).await;
    }

    let character_id = packet.read_int();
    let mac_addr = packet.read_string();
    let host_addr = packet.read_string();

    client.session.character_id = character_id;
    super::select_character::connect_to_world_server(client).await?;
    Ok(())
}

///
pub fn invalid_pic() -> Packet {
    let mut packet = Packet::new();
    packet.write_short(0x1C);
    packet.write_byte(0);
    packet
}
