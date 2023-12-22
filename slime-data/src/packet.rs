use crate::sql;
use slime_net::Packet;

/// Writes a character's "style" to a packet (gender, skin colour, face, and hair)
pub fn write_character_style(packet: &mut Packet, character: &sql::Character) {
    packet.write_byte(character.gender as u8);
    packet.write_byte(character.skin_colour as u8);
    packet.write_int(character.face);
    packet.write_byte(1); // 0: megaphone, 1: normal
    packet.write_int(character.hair);
}

/// Writes a character's equipment to a packet
pub fn write_character_equipment(packet: &mut Packet, equipment: &[sql::Equipment]) {
    for equip in equipment.iter() {
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
pub fn write_character_stats(packet: &mut Packet, character: &sql::Character) {
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

    // SP is stored as a comma seperated array, need to split the string
    // and get the value for the correct index
    let sp: i16 = character
        .sp
        .split(',')
        .collect::<Vec<_>>()
        .get(sp_index as usize)
        .unwrap()
        .parse()
        .unwrap();

    packet.write_short(sp);
    packet.write_int(character.exp);
    packet.write_short(character.fame as i16);
    packet.write_int(character.gacha_exp);
    packet.write_int(character.map);
    packet.write_byte(character.spawn_point as u8);
    packet.write_int(0);
}

pub enum SpecialEffect {
    LevelUp = 0,
    SafetyCharms = 6,
    EnterPortal = 7,
    JobChange = 8,
    QuestComplete = 9,
    Recovery = 10,
    Buff = 11,
    MonsterBookPickup = 14,
    EquipmentLevelUp = 15,
    MakerSkillSuccess = 16,
    BuffWithSound = 17,
    ExpCard = 19,
    WheelOfDestiny = 21,
    SpiritStone = 26,
}

/// Applies a special effect to the current player (shown to themselves)
pub fn show_special_effect(effect: SpecialEffect) -> Packet {
    let mut packet = Packet::new(0xCE);
    packet.write_byte(effect as u8);
    packet
}

/// Applies a special effect to the given character for everyone else in the map
pub fn show_foreign_effect(character_id: i32, effect: SpecialEffect) -> Packet {
    let mut packet = Packet::new(0xC6);
    packet.write_int(character_id);
    packet.write_byte(effect as u8);
    packet
}
