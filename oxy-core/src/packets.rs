use crate::{net::Packet, prisma::character};

///
pub fn write_character_stats(packet: &mut Packet, character: &character::Data) {
    packet.write_int(character.id);

    let mut fixed_name = String::from(&character.name);

    for _ in fixed_name.len()..13 {
        fixed_name.push('\0');
    }

    packet.write_fixed_string(&fixed_name);
    packet.write_byte(character.gender as u8);
    packet.write_byte(character.skin_colour as u8);
    packet.write_int(character.face);
    packet.write_int(character.hair);

    for i in 0..3 {
        // TODO write pet ids if exists
        packet.write_long(0);
    }

    packet.write_byte(character.level as u8);
    packet.write_short(character.job as i16);
    packet.write_short(character.str as i16);
    packet.write_short(character.dex as i16);
    packet.write_short(character.int as i16);
    packet.write_short(character.luk as i16);
    packet.write_short(character.hp as i16);
    packet.write_short(character.max_hp as i16);
    packet.write_short(character.mp as i16);
    packet.write_short(character.max_mp as i16);
    packet.write_short(character.ap as i16);

    let mut sp_index = 0;

    if character.job >= 2210 && character.job <= 2218 {
        sp_index = character.job - 2209;
    }

    packet.write_short(*character.sp.get(sp_index as usize).unwrap() as i16);
    packet.write_int(character.exp);
    packet.write_short(character.fame as i16);
    packet.write_int(character.gacha_exp);
    packet.write_int(character.map);
    packet.write_byte(character.spawn_point as u8);
    packet.write_int(0);
}
