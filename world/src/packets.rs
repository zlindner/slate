use chrono::{Local, Utc};
use oxide_core::{
    maple::{Character, Item},
    net::{packets::write_character_stats, Packet},
    util::time::current_time_ms,
};

pub fn character_info(character: &Character) -> Packet {
    let mut packet = Packet::new();
    packet.write_short(0x7D);
    packet.write_int(character.channel_id.into());
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

fn write_character(character: &Character, packet: &mut Packet) {
    packet.write_long(-1);
    packet.write_byte(0);
    write_character_stats(&character, packet);
    packet.write_byte(character.pg.buddy_capacity as u8);

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

    packet.write_int(character.pg.mesos);
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
    packet.write_byte(character.pg.equip_slots as u8);
    packet.write_byte(character.pg.use_slots as u8);
    packet.write_byte(character.pg.setup_slots as u8);
    packet.write_byte(character.pg.etc_slots as u8);
    packet.write_byte(96); // FIXME: cash slot limit

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
    packet.write_int(character.pg.monster_book_cover);
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

pub fn keymap(character: &Character) -> Packet {
    let mut packet = Packet::new();
    packet.write_short(0x14F);
    packet.write_byte(0);

    for i in 0..90 {
        match character.keymaps.get(i) {
            Some(binding) => {
                packet.write_byte(binding._type as u8);
                packet.write_int(binding.action);
            }
            None => {
                packet.write_byte(0);
                packet.write_int(0);
            }
        };
    }

    packet
}

pub fn quickmap() -> Packet {
    let mut packet = Packet::new();
    packet.write_short(0x9F);
    packet.write_byte(0);
    packet
}

// TODO
pub fn macros() -> Packet {
    let mut packet = Packet::new();
    packet.write_short(0x7C);
    packet.write_byte(0);
    packet
}

// TODO write buddies
pub fn buddy_list() -> Packet {
    let mut packet = Packet::new();
    packet.write_short(0x3F);
    packet.write_byte(7);
    packet.write_byte(0); // buddylist size
    packet
}

// TODO values are hardcoded so just writing 0 might cause issues
pub fn family_entitlements() -> Packet {
    let mut packet = Packet::new();
    packet.write_short(0x64);
    packet.write_int(0);
    packet
}

// TODO currently only writes empty family
pub fn family_info() -> Packet {
    let mut packet = Packet::new();
    packet.write_short(0x5F);
    packet.write_int(0); // current rep left
    packet.write_int(0); // total rep left
    packet.write_int(0); // todays rep
    packet.write_short(0); // juniors added
    packet.write_short(2); // juniors allowed
    packet.write_short(0);
    packet.write_int(0); // leader ID
    packet.write_string("");
    packet.write_string(""); //family message
    packet.write_int(0);
    packet
}

pub fn gender(character: &Character) -> Packet {
    let mut packet = Packet::new();
    packet.write_short(0x3A);
    packet.write_byte(character.pg.gender as u8);
    packet
}

pub fn enable_report() -> Packet {
    let mut packet = Packet::new();
    packet.write_short(0x2F);
    packet.write_byte(1);
    packet
}

pub fn enable_actions() -> Packet {
    let mut packet = Packet::new();
    packet.write_short(0x1F);
    packet.write_byte(1);
    packet.write_int(0);
    packet
}

// TODO make type an enum
// 0: [Notice] 1: Popup
// * 2: Megaphone 3: Super Megaphone 4: Scrolling message at top
// 5: Pink Text 6: Lightblue Text 7: BroadCasting NPC
pub fn server_message(_type: u8, message: String) -> Packet {
    let mut packet = Packet::new();
    packet.write_short(0x44);

    // set scrolling for server message
    if _type == 4 {
        packet.write_byte(1);
    }

    packet.write_string(&message);

    // TODO handle other types
    packet
}
