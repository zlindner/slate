use super::Packet;
use crate::maple::Character;

pub fn write_character_stats(character: &Character, packet: &mut Packet) {
    packet.write_int(character.pg.id);

    let mut padded_name = String::from(character.pg.name.clone());

    for _ in padded_name.len()..13 {
        padded_name.push('\0');
    }

    packet.write_fixed_string(&padded_name);

    // style
    packet.write_byte(character.pg.gender as u8);
    packet.write_byte(character.pg.skin_colour as u8);
    packet.write_int(character.pg.face);
    packet.write_int(character.pg.hair);

    // pets
    for i in 0..3 {
        match character.pets.get(i) {
            Some(pet) => packet.write_long(pet.id.into()),
            None => packet.write_long(0),
        }
    }

    // stats
    packet.write_byte(character.pg.level as u8);
    packet.write_short(character.pg.job);
    packet.write_short(character.pg.str);
    packet.write_short(character.pg.dex);
    packet.write_short(character.pg.int);
    packet.write_short(character.pg.luk);
    packet.write_short(character.pg.hp);
    packet.write_short(character.pg.max_hp);
    packet.write_short(character.pg.mp);
    packet.write_short(character.pg.max_mp);
    packet.write_short(character.pg.ap);
    // TODO can add remaining skill info here for evan
    // TODO get characters remaining sp for job
    packet.write_short(0);
    packet.write_int(character.pg.exp);
    packet.write_short(character.pg.fame as i16);
    packet.write_int(character.pg.gacha_exp);
    packet.write_int(character.pg.map);
    packet.write_byte(character.pg.spawn_point as u8);
    packet.write_int(0);
}
