use crate::{model::LoginState, query, server::LoginSession};
use slime_net::Packet;

/// Login server: select character packet (0x13)
/// Called when the client selects a character and doesn't have a PIC
pub async fn handle(mut packet: Packet, session: &mut LoginSession) -> anyhow::Result<()> {
    let character_id = packet.read_int();
    let mac_addr = packet.read_string();
    let host_addr = packet.read_string();

    session.data.character_id = character_id;
    connect_to_world_server(session).await?;
    Ok(())
}

///
pub async fn connect_to_world_server(session: &mut LoginSession) -> anyhow::Result<()> {
    // TODO we can check mac_addr/hwid from host_addr if we want to prevent multi-logging

    sqlx::query(
        "INSERT INTO sessions (id, account_id, character_id, world_id, channel_id, map_id) 
        VALUES (?, ?, ?, ?, ?, ?)",
    )
    .bind(session.id)
    .bind(session.data.account_id)
    .bind(session.data.character_id)
    .bind(session.data.world_id)
    .bind(session.data.channel_id)
    .bind(session.data.map_id)
    .execute(&session.db)
    .await?;

    query::update_login_state(session, LoginState::Transitioning).await?;

    session
        .stream
        .write_packet(world_server_addr(session))
        .await?;

    Ok(())
}

/// Packet containing the world server addr and client's session id
fn world_server_addr(session: &LoginSession) -> Packet {
    let mut packet = Packet::new(0x0C);
    packet.write_short(0);

    // Get the world server ip and convert each "." delimited section to a u8
    let ip = std::env::var("WORLD_IP").unwrap_or("0.0.0.0".to_string());
    let ip = ip.split('.').collect::<Vec<&str>>();
    packet.write_bytes(&[
        ip.first().unwrap().parse::<u8>().unwrap(),
        ip.get(1).unwrap().parse::<u8>().unwrap(),
        ip.get(2).unwrap().parse::<u8>().unwrap(),
        ip.get(3).unwrap().parse::<u8>().unwrap(),
    ]);

    let world = session
        .config
        .worlds
        .get(session.data.world_id as usize)
        .unwrap();

    packet.write_short((world.base_port + session.data.channel_id) as i16);

    // NOTE: this is technically supposed to be the character id, but we need
    // some way to tell the world server the client's session id.
    packet.write_int(session.id);
    packet.write_bytes(&[0, 0, 0, 0, 0]);
    packet
}
