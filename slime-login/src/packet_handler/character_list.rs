use super::world_status::{self, WorldStatus};
use crate::{packet, server::LoginSession};
use slime_data::{maple, sql};
use slime_net::Packet;

/// Login server: character list packet (0x05)
/// Displays the user's character list after selecting a world and channel
pub async fn handle(mut packet: Packet, session: &mut LoginSession) -> anyhow::Result<()> {
    packet.skip(1);
    let world_id = packet.read_byte() as i32;

    let world_config = match session.config.worlds.get(world_id as usize) {
        Some(config) => config,
        None => {
            return session
                .stream
                .write_packet(world_status::world_status(WorldStatus::Full))
                .await;
        }
    };

    let world = sql::World::load(world_id, &session.db).await?;
    let channel_id = packet.read_byte() as i32;
    log::debug!("CHANNEL_ID {}", channel_id);

    // Check if the selected world is already full or if the channel is invalid
    if world.connected_players >= world_config.max_players || channel_id >= world_config.channels {
        return session
            .stream
            .write_packet(world_status::world_status(WorldStatus::Full))
            .await;
    }

    session.data.world_id = world_id;
    session.data.channel_id = channel_id;

    // Load the current account's characters in the selected world
    let sql_characters =
        sql::Character::load_all(session.data.account_id, world_id, &session.db).await?;

    let mut maple_characters = Vec::with_capacity(sql_characters.len());

    for sql_character in sql_characters.into_iter() {
        // Load the equipment for each character
        let equipment = sql::Equipment::load_all(sql_character.id, &session.db).await?;

        // Build the maple character
        maple_characters.push(maple::Character {
            data: sql_character,
            equipment,
            items: Vec::new(), // items aren't needed for character list
        });
    }

    session
        .stream
        .write_packet(character_list(maple_characters, session))
        .await?;

    Ok(())
}

/// Character list packet, contains stat, style, and equipment data for each
/// character in the selected world
fn character_list(characters: Vec<maple::Character>, session: &LoginSession) -> Packet {
    let mut packet = Packet::new(0x0B);
    packet.write_byte(0); // status
    packet.write_byte(characters.len() as u8);

    for character in characters.iter() {
        write_character(&mut packet, character, false);
    }

    // 0: register pic (user hasn't registered pic)
    // 1: prompt pic (user already registered pic)
    // 2: disable pic
    let pic_flag = match session.config.enable_pic {
        true => !session.data.pic.is_empty() as u8,
        false => 2,
    };

    packet.write_byte(pic_flag);
    packet.write_int(3); // character slots
    packet
}

/// Writes a character's stat, style, and equipment data to a packet
pub fn write_character(packet: &mut Packet, character: &maple::Character, view_all: bool) {
    packet::write_character_stats(packet, character);
    packet::write_character_style(packet, character);
    packet::write_character_equipment(packet, character);

    if !view_all {
        packet.write_byte(0);
    }

    let job_niche = (character.data.job / 100) % 10;

    if character.data.gm > 1 || job_niche == 8 || job_niche == 9 {
        packet.write_byte(0);
        return;
    }

    // world rank enabled, following 4 ints aren't sent if disabled
    packet.write_byte(1);
    packet.write_int(character.data.rank);
    packet.write_int(character.data.rank_move);
    packet.write_int(character.data.job_rank);
    packet.write_int(character.data.job_rank_move);
}
