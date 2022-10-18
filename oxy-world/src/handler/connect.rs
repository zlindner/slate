use anyhow::Result;
use oxy_core::net::{Client, Packet};

/// World server: connect packet (0x14)
/// Called when the client begins transition from login -> world server
pub async fn handle(mut packet: Packet, client: &mut Client) -> Result<()> {
    Ok(())
}
