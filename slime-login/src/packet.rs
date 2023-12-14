use slime_data::maple;
use slime_net::Packet;

/// Writes a character's "style" to a packet (gender, skin colour, face, and hair)
pub fn write_character_style(packet: &mut Packet, character: &maple::Character) {
    packet.write_byte(character.data.gender as u8);
    packet.write_byte(character.data.skin_colour as u8);
    packet.write_int(character.data.face);
    packet.write_byte(1); // 0: megaphone, 1: normal
    packet.write_int(character.data.hair);
}

/// Writes a character's equipment to a packet
pub fn write_character_equipment(packet: &mut Packet, character: &maple::Character) {
    for equip in character.equipment.iter() {
        packet.write_byte(equip.position as u8);
        packet.write_int(equip.item_id);
    }

    packet.write_byte(0xFF);
    // TODO write masked equips
    packet.write_byte(0xFF);
    // TODO write item @ pos -111 (weapon?)
    packet.write_int(0);

    for _ in 0..3 {
        // TODO write pet item id's
        packet.write_int(0);
    }
}

/// Writes a character's stats to a packet
pub fn write_character_stats(packet: &mut Packet, character: &maple::Character) {
    packet.write_int(character.data.id);

    let mut fixed_name = String::from(&character.data.name);

    for _ in fixed_name.len()..13 {
        fixed_name.push('\0');
    }

    packet.write_fixed_string(&fixed_name);
    packet.write_byte(character.data.gender as u8);
    packet.write_byte(character.data.skin_colour as u8);
    packet.write_int(character.data.face);
    packet.write_int(character.data.hair);

    for i in 0..3 {
        // TODO write pet ids if exists
        packet.write_long(0);
    }

    packet.write_byte(character.data.level as u8);
    packet.write_short(character.data.job as i16);
    packet.write_short(character.data.str as i16);
    packet.write_short(character.data.dex as i16);
    packet.write_short(character.data.int as i16);
    packet.write_short(character.data.luk as i16);
    packet.write_short(character.data.hp as i16);
    packet.write_short(character.data.max_hp as i16);
    packet.write_short(character.data.mp as i16);
    packet.write_short(character.data.max_mp as i16);
    packet.write_short(character.data.ap as i16);

    let mut sp_index = 0;

    if character.data.job >= 2210 && character.data.job <= 2218 {
        sp_index = character.data.job - 2209;
    }

    // SP is stored as a comma seperated array, need to split the string
    // and get the value for the correct index
    let sp: i16 = character
        .data
        .sp
        .split(',')
        .collect::<Vec<_>>()
        .get(sp_index as usize)
        .unwrap()
        .parse()
        .unwrap();

    packet.write_short(sp);
    packet.write_int(character.data.exp);
    packet.write_short(character.data.fame as i16);
    packet.write_int(character.data.gacha_exp);
    packet.write_int(character.data.map);
    packet.write_byte(character.data.spawn_point as u8);
    packet.write_int(0);
}
