use crate::Character;

use super::Packet;

pub fn write_character_stats(character: &Character, packet: &mut Packet) {
    packet.write_int(character.id);

    let mut padded_name = String::from(character.name.clone());

    for _ in padded_name.len()..13 {
        padded_name.push('\0');
    }

    packet.write_fixed_string(&padded_name);

    // style
    packet.write_byte(character.gender as u8);
    packet.write_byte(character.skin_colour as u8);
    packet.write_int(character.face);
    packet.write_int(character.hair);

    // pets
    for i in 0..3 {
        match character.pets.get(i) {
            Some(pet) => packet.write_long(pet.id.into()),
            None => packet.write_long(0),
        }
    }

    // stats
    packet.write_byte(character.level as u8);
    packet.write_short(character.job);
    packet.write_short(character.str);
    packet.write_short(character.dex);
    packet.write_short(character.int);
    packet.write_short(character.luk);
    packet.write_short(character.hp);
    packet.write_short(character.max_hp);
    packet.write_short(character.mp);
    packet.write_short(character.max_mp);
    packet.write_short(character.ap);
    // TODO can add remaining skill info here for evan
    // TODO get characters remaining sp for job
    packet.write_short(0);
    packet.write_int(character.exp);
    packet.write_short(character.fame as i16);
    packet.write_int(character.gacha_exp);
    packet.write_int(character.map);
    packet.write_byte(character.spawn_point as u8);
    packet.write_int(0);
}
