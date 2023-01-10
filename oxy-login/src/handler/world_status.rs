use crate::{client::LoginClient, shared::Shared};
use anyhow::Result;
use oxy_core::{net::Packet, prisma::world};

/// Login server: world status (0x06)
/// Displays the selected world's status/capacity for each channel
/// NOTE: the same status bar is shown for each channel
pub async fn handle(mut packet: Packet, client: &mut LoginClient, shared: &Shared) -> Result<()> {
    let world_id = packet.read_short();

    let response = match shared.config.worlds.get(world_id as usize) {
        None => world_status(WorldStatus::Full),
        Some(world_config) => {
            let world = client
                .db
                .world()
                .find_unique(world::id::equals(world_id as i32))
                .exec()
                .await?
                .unwrap();

            // TODO add max_players_channel? we can do world_config.channels * max_players_channel instead
            let status = if world.connected >= world_config.max_players {
                WorldStatus::Full
            } else if world.connected >= ((world_config.max_players as f32) * 0.8) as i32 {
                WorldStatus::HighlyPopulated
            } else {
                WorldStatus::Normal
            };

            world_status(status)
        }
    };

    client.send(response).await?;
    Ok(())
}

pub enum WorldStatus {
    Normal,
    HighlyPopulated,
    Full,
}

/// Packet containing the world's capacity status
pub fn world_status(status: WorldStatus) -> Packet {
    let mut packet = Packet::new();
    packet.write_short(0x03);
    packet.write_short(status as i16);
    packet
}
