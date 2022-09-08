use chrono::{Local, Utc};
use oxide_core::{
    maple::{Character, Item},
    net::{packets::write_character_stats, Packet},
    util::time::current_time_ms,
};

pub fn character_info(character: Character) -> Packet {
    let mut packet = Packet::new();
    packet.write_short(0x7D);
    packet.write_int(0); // FIXME channel
    packet.write_byte(1);
    packet.write_byte(1);
    packet.write_short(0);
    packet.write_int(rand::random());
    packet.write_int(rand::random());
    packet.write_int(rand::random());
    write_character(character, &mut packet);

    let current_time = Utc::now().timestamp_millis() * 10000;
    let offset: i64 =
        116444736010800000 + (10000000 * i64::from(Local::now().offset().local_minus_utc()));

    packet.write_long(current_time + offset);
    packet
}

fn write_character(character: Character, packet: &mut Packet) {
    packet.write_long(-1);
    packet.write_byte(0);
    write_character_stats(&character, packet);
    packet.write_byte(10); // FIXME characters buddy list capacity

    // TODO blessing of the fairy stuff?
    packet.write_byte(0);
    /*
    if character.linked_name == null {
        packet.write_byte(0);
    } else {
        packet.write_byte(1);
        packet.write_string(character.linked_name);
    }
    */

    packet.write_int(character.mesos);
    write_character_inventory(&character, packet);
    write_character_skills(&character, packet);
    write_character_quests(&character, packet);
    packet.write_short(0); // TODO minigame info?
    write_character_rings(&character, packet);
    write_character_teleport_rock_maps(&character, packet);
    write_character_monster_book(&character, packet);
    packet.write_short(0); // TODO new year card info
    packet.write_short(0); // TODO area info (not sure this is)
    packet.write_short(0);
}

fn write_character_inventory(character: &Character, packet: &mut Packet) {
    for _ in 0..5 {
        // TODO get slot limit for each inventory type
        packet.write_byte(10);
    }

    // UTC zero-timestamp
    packet.write_long(94354848000000000);

    // TODO equipped items
    packet.write_short(0);
    // TODO equipped cash items
    packet.write_short(0);

    for item in character.equip_inventory.values() {
        write_item(item, packet);
    }

    packet.write_int(0);

    for item in character.use_inventory.values() {
        write_item(item, packet);
    }

    packet.write_byte(0);

    for item in character.setup_inventory.values() {
        write_item(item, packet);
    }

    packet.write_byte(0);

    for item in character.etc_inventory.values() {
        write_item(item, packet);
    }

    packet.write_byte(0);

    for item in character.cash_inventory.values() {
        write_item(item, packet);
    }
}

fn write_character_skills(character: &Character, packet: &mut Packet) {
    packet.write_byte(0);

    // TODO skip "hidden" skills

    packet.write_short(character.skills.len() as i16);

    for skill in character.skills.iter() {
        packet.write_int(skill.skill_id);
        packet.write_int(skill.level);
        packet.write_long(skill.expiration); // FIXME cosmic uses some really weird normalization here

        if skill.is_fourth_job() {
            packet.write_int(skill.mastery_level);
        }
    }

    packet.write_short(character.cooldowns.len() as i16);

    for cooldown in character.cooldowns.iter() {
        packet.write_int(cooldown.skill_id);

        let remaining = cooldown.start_time + cooldown.length - current_time_ms();
        packet.write_short((remaining / 1000) as i16);
    }
}

fn write_character_quests(character: &Character, packet: &mut Packet) {
    // TODO in progress quests
    packet.write_short(0);
    // TODO completed quests
    packet.write_short(0);
}

fn write_character_rings(character: &Character, packet: &mut Packet) {
    // TODO crush rings
    packet.write_short(0);
    // TODO friendship rings
    packet.write_short(0);
    // TODO marriage rings
    packet.write_short(0);
}

fn write_character_teleport_rock_maps(character: &Character, packet: &mut Packet) {
    // TODO teleport rock maps
    for _ in 0..5 {
        packet.write_int(999999999);
    }

    // TODO vip teleport rock maps
    for _ in 0..10 {
        packet.write_int(999999999);
    }
}

fn write_character_monster_book(character: &Character, packet: &mut Packet) {
    packet.write_int(character.monster_book_cover);
    packet.write_byte(0);
    // FIXME cards.len()
    packet.write_short(0);
    // TODO write each monster card
}

fn write_item(item: &Item, packet: &mut Packet) {
    // TODO pass in?
    let zero_pos = false;
    let mut pos = item.position;

    if !zero_pos {
        if item.item_type == 1 {
            if item.position < 0 {
                pos *= -1;
            }

            if pos > 100 {
                pos -= 100;
            }

            packet.write_short(pos);
        } else {
            packet.write_byte(pos as u8);
        }
    }

    packet.write_byte(item.item_type);
    packet.write_int(item.id);
    packet.write_byte(item.is_cash() as u8);

    if item.is_cash() {
        // TODO if pet write pet id, if ring write ring id, otherwise write cash id
        // can we just write the id here?
    }

    // TODO expiration time

    // TODO if item.is_pet()
}
