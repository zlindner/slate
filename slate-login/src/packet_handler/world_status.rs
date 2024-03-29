use crate::server::LoginSession;
use slate_net::Packet;
use sqlx::Row;

/// Login server: world status (0x06)
/// Displays the selected world's status/capacity for each channel
/// NOTE: the same status bar is shown for each channel
pub async fn handle(mut packet: Packet, session: &mut LoginSession) -> anyhow::Result<()> {
    let world_id = packet.read_short();

    let response = match session.config.worlds.get(world_id as usize) {
        None => world_status(WorldStatus::Full),
        Some(world_config) => {
            let players: u64 = sqlx::query(
                "SELECT CAST(SUM(connected_players) AS UNSIGNED) AS sum FROM channels WHERE world_id = 1",
            )
            .bind(world_id)
            .fetch_one(&session.db)
            .await?
            .get("sum");

            // FIXME
            let players: i32 = players.try_into().unwrap();

            // TODO add max_players_channel? we can do world_config.channels * max_players_channel instead
            let status = if players >= world_config.max_players {
                WorldStatus::Full
            } else if players >= ((world_config.max_players as f32) * 0.8) as i32 {
                WorldStatus::HighlyPopulated
            } else {
                WorldStatus::Normal
            };

            world_status(status)
        }
    };

    session.stream.write_packet(response).await?;
    Ok(())
}

pub enum WorldStatus {
    Normal,
    HighlyPopulated,
    Full,
}

/// Packet containing the world's capacity status
pub fn world_status(status: WorldStatus) -> Packet {
    let mut packet = Packet::new(0x03);
    packet.write_short(status as i16);
    packet
}
