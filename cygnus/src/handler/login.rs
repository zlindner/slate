use crate::client::CygnusClient;
use anyhow::Result;
use oxy_core::net::Packet;

/// Client: login result packet (0x00)
/// Handles the login response from the server after clicking login
pub async fn handle(mut packet: Packet, client: &mut CygnusClient) -> Result<()> {
    let reason = packet.read_int();

    match reason {
        0 => log::info!("Success"),
        3 => log::info!("Account banned"),
        5 => log::info!("Account not found"),
        6 => log::info!("Too many attempts"),
        7 => log::info!("Already logged in"),
        23 => log::info!("Prompting tos..."),
        _ => log::error!("Unexpected reason code: {}", reason),
    }

    Ok(())
}
