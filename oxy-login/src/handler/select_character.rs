use crate::client::LoginClient;
use anyhow::Result;
use oxy_core::{net::Packet, prisma::LoginState};

/// Login server: select character packet (0x13)
/// Called when the client selects a character and doesn't have a PIC
pub async fn handle(mut packet: Packet, client: &mut LoginClient) -> Result<()> {
    let character_id = packet.read_int();
    let mac_addr = packet.read_string();
    let host_addr = packet.read_string();

    client.session.character_id = character_id;

    Ok(())
}

pub async fn connect_to_world_server(client: &mut LoginClient) -> Result<()> {
    // TODO we can check mac_addr/hwid from host_addr if we want to prevent multi-logging

    client
        .db
        .session()
        .create(
            client.session.id,
            client.session.account_id,
            client.session.character_id,
            client.session.world_id,
            client.session.channel_id,
            client.session.login_attempts,
            client.session.pin.clone(),
            client.session.pin_attempts,
            client.session.pic.clone(),
            client.session.pic_attempts,
            client.session.tos,
            vec![],
        )
        .exec()
        .await?;

    // TODO is this really required?
    client.update_login_state(LoginState::Transitioning).await?;

    let response = world_server_addr(client.session.id);
    client.send(response).await?;
    Ok(())
}

/// Packet containing the world server addr and client's session id
fn world_server_addr(session_id: i32) -> Packet {
    let mut packet = Packet::new();
    packet.write_short(0x0C);
    packet.write_short(0);

    // Get the world server ip and convert each "." delimited section to a u8
    let ip = std::env::var("WORLD_IP").unwrap_or("0.0.0.0".to_string());
    let ip = ip.split(".").collect::<Vec<&str>>();
    let ip_bytes = [
        ip.get(0).unwrap().parse::<u8>().unwrap(),
        ip.get(1).unwrap().parse::<u8>().unwrap(),
        ip.get(2).unwrap().parse::<u8>().unwrap(),
        ip.get(3).unwrap().parse::<u8>().unwrap(),
    ];

    packet.write_bytes(&ip_bytes);
    // FIXME correct port for selected channel
    // TODO this should probably be in config/env?
    // TODO can we handle all channels on a single port?
    packet.write_short(10000);

    // NOTE: this is technically supposed to be the character id, but we need
    // some way to tell the world server the client's session id.
    packet.write_int(session_id);
    packet.write_bytes(&[0, 0, 0, 0, 0]);
    packet
}
