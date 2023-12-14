use crate::server::LoginSession;
use slime_data::config;
use slime_net::Packet;

/// Login server: world list (0x0B)
/// Displays the world list to the user after successful login
pub async fn handle(_packet: Packet, session: &mut LoginSession) -> anyhow::Result<()> {
    for world in session.config.worlds.iter() {
        session.stream.write_packet(world_info(world)).await?;
    }

    session.stream.write_packet(world_list_end()).await?;

    // NOTE: We can send a 0x1A packet here to pre-select the most active world,
    // but that doesn't seem like great a UX and some clients ignore it anyway
    session
        .stream
        .write_packet(recommended_world(&session.config.worlds))
        .await?;

    Ok(())
}

/// Packet containing info for each world (name, rates, etc.)
fn world_info(world: &config::World) -> Packet {
    let mut packet = Packet::new(0x0A);
    packet.write_byte(world.id as u8);
    packet.write_string(&world.name);
    packet.write_byte(world.flag as u8); // 0: normal, 1: event, 2: new, 3: hot
    packet.write_string(&world.event_message);
    packet.write_bytes(&[100, 0, 100, 0, 0]);
    packet.write_byte(world.channels as u8);

    for i in 0..world.channels {
        let name = format!("{}{}", world.name, i);
        packet.write_string(&name);
        // TODO channel capacity: connected(?) characters / channel_load (100?) * 800
        // TODO make max players per channel configurable
        packet.write_int(100);
        packet.write_byte(world.id as u8);
        packet.write_byte(i as u8);
        packet.write_byte(0); // Adult channel flag
    }

    packet.write_short(0);
    packet
}

/// Packet indicating we are finished sending world_info packets
fn world_list_end() -> Packet {
    let mut packet = Packet::new(0x0A);
    packet.write_byte(0xFF);
    packet
}

/// Packet containing recommended message for each world
fn recommended_world(worlds: &Vec<config::World>) -> Packet {
    let mut packet = Packet::new(0x1B);
    packet.write_byte(worlds.len() as u8);

    for world in worlds.iter() {
        packet.write_int(world.id);
        packet.write_string(&world.recommended_message);
    }

    packet
}
