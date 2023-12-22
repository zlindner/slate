use crate::server::LoginSession;
use slime_data::sql::{self, account::LoginState};
use slime_net::Packet;
use std::env;

/// Login server: select character packet (0x13)
/// Called when the client selects a character and doesn't have a PIC
pub async fn handle(mut packet: Packet, session: &mut LoginSession) -> anyhow::Result<()> {
    let character_id = packet.read_int();
    let mac_addr = packet.read_string();
    let host_addr = packet.read_string();

    session.data.character_id = character_id;
    connect_to_channel_server(session).await?;
    Ok(())
}

///
pub async fn connect_to_channel_server(session: &mut LoginSession) -> anyhow::Result<()> {
    // TODO we can check mac_addr/hwid from host_addr if we want to prevent multi-logging

    sqlx::query(
        "INSERT INTO login_sessions (id, account_id, character_id, world_id, channel_id) 
        VALUES (?, ?, ?, ?, ?)",
    )
    .bind(session.id)
    .bind(session.data.account_id)
    .bind(session.data.character_id)
    .bind(session.data.world_id)
    .bind(session.data.channel_id)
    .execute(&session.db)
    .await?;

    // Indicate we are transition to the login server -- don't update login state when session closes
    session.transitioning = true;
    sql::Account::update_login_state(
        session.data.account_id,
        LoginState::Transitioning,
        &session.db,
    )
    .await?;

    session
        .stream
        .write_packet(channel_server_addr(session))
        .await?;

    sqlx::query(
        "UPDATE channels SET connected_players = connected_players + 1 WHERE world_id = ? AND id = ?"
    )
    .bind(session.data.world_id)
    .bind(session.data.channel_id)
    .execute(&session.db)
    .await?;

    Ok(())
}

/// Packet containing the channel server addr and client's session id
fn channel_server_addr(session: &LoginSession) -> Packet {
    let mut packet = Packet::new(0x0C);
    packet.write_short(0);

    // Get the channel server ip and convert each "." delimited section to a u8
    let ip = env::var("CHANNEL_IP").expect("Channel ip should be defined in .env");
    let ip = ip.split('.').collect::<Vec<&str>>();
    packet.write_bytes(&[
        ip.first().unwrap().parse::<u8>().unwrap(),
        ip.get(1).unwrap().parse::<u8>().unwrap(),
        ip.get(2).unwrap().parse::<u8>().unwrap(),
        ip.get(3).unwrap().parse::<u8>().unwrap(),
    ]);

    let base_port: i32 = env::var("CHANNEL_BASE_PORT")
        .expect("Channel base port should be defined in .env")
        .parse()
        .expect("Channel base port should be a valid integer");

    let port = base_port + (session.data.world_id * 1000) + (session.data.channel_id);
    packet.write_short(port as i16);

    // NOTE: this is technically supposed to be the character id, but we need
    // some way to tell the world server the client's session id.
    packet.write_int(session.id);
    packet.write_bytes(&[0, 0, 0, 0, 0]);
    packet
}
