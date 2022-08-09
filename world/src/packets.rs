use oxide_core::{
    net::{packets::write_character_stats, Packet},
    Character,
};
use std::time::{SystemTime, UNIX_EPOCH};

pub fn character_info(character: Character) -> Packet {
    let mut packet = Packet::new();
    packet.write_int(0); // FIXME channel
    packet.write_byte(1);
    packet.write_byte(1);
    packet.write_short(0);
    packet.write_int(rand::random());
    packet.write_int(rand::random());
    packet.write_int(rand::random());

    write_character(character, &mut packet);

    packet.write_long(
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis() as i64,
    );

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
    // inventory info
    write_character_skills(character, packet);
    // quest info
    // minigame info
    // ring info
    // teleport info
    // monster book info
    // new year info
    // area info
    packet.write_short(0);
}

fn write_character_skills(character: Character, packet: &mut Packet) {
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
}
