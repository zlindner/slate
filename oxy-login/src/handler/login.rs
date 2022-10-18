use anyhow::Result;
use oxy_core::net::{Client, Packet};

/// Login server: login packet (0x01)
/// Called when the client clicks login after entering name and password
pub async fn handle(mut packet: Packet, client: &mut Client) -> Result<()> {
    let name = packet.read_string();
    let password = packet.read_string();
    packet.skip(6);
    let hwid = packet.read_bytes(4);

    log::debug!("Name: {}", name);
    log::debug!("Password: {}", password);

    Ok(())
}
