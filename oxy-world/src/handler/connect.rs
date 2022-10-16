use anyhow::Result;
use oxy_core::{net::Packet, prisma::PrismaClient};
use std::sync::Arc;

/// World server: connect packet (0x14)
/// Called when the client begins transition from login -> world server
pub async fn handle(mut packet: Packet, db: &Arc<PrismaClient>) -> Result<()> {
    Ok(())
}
