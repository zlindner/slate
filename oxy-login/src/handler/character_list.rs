use super::{
    world_status::{self, WorldStatus},
    Config,
};
use anyhow::Result;
use oxy_core::{
    net::{Client, Packet},
    packets,
    prisma::{character, world},
};

/// Login server: character list packet (0x05)
/// Displays the user's character list after selecting a world and channel
pub async fn handle(mut packet: Packet, client: &mut Client, config: &Config) -> Result<()> {
    packet.skip(1);
    let world_id = packet.read_byte();

    let world_config = match config.worlds.get(world_id as usize) {
        Some(config) => config,
        None => {
            let response = world_status::world_status(WorldStatus::Full);
            return client.send(response).await;
        }
    };

    let world = client
        .db
        .world()
        .find_unique(world::id::equals(world_id as i32))
        .exec()
        .await?
        .unwrap();

    if world.connected >= world_config.max_players {
        let response = world_status::world_status(WorldStatus::Full);
        return client.send(response).await;
    }

    let channel_id = packet.read_byte();

    if channel_id >= world_config.channels {
        let response = world_status::world_status(WorldStatus::Full);
        return client.send(response).await;
    }

    client.session.world_id = world_id as i32;
    client.session.channel_id = channel_id as i32;

    let characters = client
        .db
        .character()
        .find_many(vec![
            character::account_id::equals(client.session.account_id),
            character::world_id::equals(world_id as i32),
        ])
        .with(character::equips::fetch(vec![]))
        .exec()
        .await?;

    let response = character_list(characters, client, config);
    client.send(response).await?;
    Ok(())
}

///
fn character_list(characters: Vec<character::Data>, client: &Client, config: &Config) -> Packet {
    let mut packet = Packet::new();
    packet.write_short(0x0B);
    packet.write_byte(0); // status
    packet.write_byte(characters.len() as u8);

    for character in characters.iter() {
        write_character(&mut packet, character, false);
    }

    // 0: register pic (user hasn't registered pic)
    // 1: prompt pic (user already registered pic)
    // 2: disable pic
    let pic_flag = match config.enable_pic {
        true => !client.session.pic.is_empty() as u8,
        false => 2,
    };

    packet.write_byte(pic_flag);
    packet.write_int(3); // character slots
    packet
}

///
pub fn write_character(packet: &mut Packet, character: &character::Data, view_all: bool) {
    packets::write_character_stats(packet, character);
    write_character_style(packet, character);
    write_character_equipment(packet, character);

    if !view_all {
        packet.write_byte(0);
    }

    let job_niche = (character.job / 100) % 10;

    if character.gm > 1 || job_niche == 8 || job_niche == 9 {
        packet.write_byte(0);
        return;
    }

    packet.write_byte(1); // world rank enabled, following 4 ints aren't sent if disabled
    packet.write_int(character.rank);
    packet.write_int(character.rank_move);
    packet.write_int(character.job_rank);
    packet.write_int(character.job_rank_move);
}

///
fn write_character_style(packet: &mut Packet, character: &character::Data) {
    packet.write_byte(character.gender as u8);
    packet.write_byte(character.skin_colour as u8);
    packet.write_int(character.face);
    packet.write_byte(1); // 0: megaphone, 1: normal
    packet.write_int(character.hair);
}

///
fn write_character_equipment(packet: &mut Packet, character: &character::Data) {
    for equip in character.equips.as_ref().unwrap().iter() {
        packet.write_byte(equip.position as u8);
        packet.write_int(equip.item_id);
    }

    packet.write_byte(0xFF);
    // TODO write masked equips
    packet.write_byte(0xFF);
    // TODO write item @ pos -111 (weapon?)
    packet.write_int(0);

    for i in 0..3 {
        // TODO write pet item id's
        packet.write_int(0);
    }
}
