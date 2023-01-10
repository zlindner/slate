use crate::{character::Character, client::WorldClient, Shared};
use anyhow::Result;
use oxy_core::{
    net::Packet,
    packets,
    prisma::{character, equip, item, quest, session, InventoryType, QuestStatus},
};
use prisma_client_rust::chrono::{Local, Utc};
use rand::random;
use std::{collections::HashMap, sync::Arc};

/// World server: connect packet (0x14)
/// Called when the client begins transition from login -> world server
pub async fn handle(
    mut packet: Packet,
    client: &mut WorldClient,
    shared: &Arc<Shared>,
) -> Result<()> {
    let session_id = packet.read_int();

    let session = client
        .db
        .session()
        .find_unique(session::id::equals(session_id))
        .exec()
        .await?;

    client.session = match session {
        Some(session) => session,
        None => {
            let response = connect_error(ConnectError::Unknown);
            return client.send(response).await;
        }
    };

    let character_data = client
        .db
        .character()
        .find_unique(character::id::equals(client.session.character_id))
        .with(character::items::fetch(vec![]))
        .with(character::equips::fetch(vec![]))
        .with(character::skills::fetch(vec![]))
        .with(character::keymaps::fetch(vec![]))
        .with(character::cooldowns::fetch(vec![]))
        .with(character::quests::fetch(vec![]))
        .exec()
        .await?;

    let character_data = match character_data {
        Some(character_data) => character_data,
        None => {
            let response = connect_error(ConnectError::Unknown);
            return client.send(response).await;
        }
    };

    // Set the client's character id
    // TODO is it possible to set client.character here and insert &client.character into shared?
    let character = Character::new(character_data);
    client.map_id = character.map_id;
    client.character_id = character.id;

    let response = character_info(client.session.channel_id, &character.data);
    client.send(response).await?;

    let response = character_keymap(&character.data);
    client.send(response).await?;

    // Send the character data to all other clients
    let response = spawn_character(&character, true);
    client.broadcast(response, false).await?;

    let map = shared.get_map(character.map_id);

    let mut objects = Vec::new();

    for map_character in map.characters.iter() {
        objects.push(spawn_character(&map_character, false));
    }

    // TODO npcs, mobs, etc.

    map.characters.insert(character.id, character);

    for spawn_packet in objects.into_iter() {
        client.send(spawn_packet).await?;
    }

    Ok(())
}

enum ConnectError {
    Unknown = 0x09,
}

///
fn connect_error(error: ConnectError) -> Packet {
    let mut packet = Packet::new();
    packet.write_short(0x09);
    packet.write_short(error as i16);
    packet
}

///
fn character_info(channel_id: i32, character: &character::Data) -> Packet {
    let mut packet = Packet::new();
    packet.write_short(0x7D);
    packet.write_int(channel_id);
    packet.write_byte(1);
    packet.write_byte(1);
    packet.write_short(0);
    packet.write_int(rand::random());
    packet.write_int(rand::random());
    packet.write_int(rand::random());
    write_character(&mut packet, character);

    // FIXME this is ugly
    let current_time = Utc::now().timestamp_millis() * 10000;
    let offset: i64 =
        116444736010800000 + (10000000 * i64::from(Local::now().offset().local_minus_utc()));

    packet.write_long(current_time + offset);
    packet
}

///
fn write_character(packet: &mut Packet, character: &character::Data) {
    packet.write_long(-1);
    packet.write_byte(0);
    packets::write_character_stats(packet, character);
    packet.write_byte(character.buddy_capacity as u8);

    // TODO blessing of the fairy stuff
    packet.write_byte(0);

    packet.write_int(character.mesos);
    write_character_inventory(packet, character);
    write_character_skills(packet, character);
    write_character_quests(packet, character);
    packet.write_short(0); // TODO minigame info?
    write_character_rings(packet, character);
    write_character_teleport_rock_maps(packet, character);
    write_character_monster_book(packet, character);
    packet.write_short(0); // TODO new year card info?
    packet.write_short(0); // TODO area info?
    packet.write_short(0);
}

/// Writes a character's inventory data to a packet
fn write_character_inventory(packet: &mut Packet, character: &character::Data) {
    for slot_limit in character.slots.iter() {
        packet.write_byte(*slot_limit as u8);
    }

    // UTC zero-timestamp
    packet.write_long(94354848000000000);

    // Write equips
    if let Some(equips) = character.equips.as_ref() {
        for equip in equips.iter() {
            write_equip(packet, equip);
        }
    }

    packet.write_short(0);
    // TODO write equipped cash items
    packet.write_short(0);

    let mut inventory: HashMap<i32, Vec<&item::Data>> = HashMap::new();

    if let Some(items) = character.items.as_ref() {
        for item in items.iter() {
            if !inventory.contains_key(&(item.inventory_type as i32)) {
                inventory.insert(item.inventory_type as i32, Vec::new());
            }

            inventory
                .get_mut(&(item.inventory_type as i32))
                .unwrap()
                .push(item);
        }
    }

    // Write equip inventory
    if let Some(equip_inventory) = inventory.get(&(InventoryType::Equip as i32)) {
        for item in equip_inventory.iter() {
            write_item(packet, item);
        }
    }

    packet.write_int(0);

    // Write use inventory
    if let Some(use_inventory) = inventory.get(&(InventoryType::Use as i32)) {
        for item in use_inventory.iter() {
            write_item(packet, item);
        }
    }

    packet.write_byte(0);

    // Write setup inventory
    if let Some(setup_inventory) = inventory.get(&(InventoryType::Setup as i32)) {
        for item in setup_inventory.iter() {
            write_item(packet, item);
        }
    }

    packet.write_byte(0);

    // Write etc inventory
    if let Some(etc_inventory) = inventory.get(&(InventoryType::Etc as i32)) {
        for item in etc_inventory.iter() {
            write_item(packet, item);
        }
    }

    packet.write_byte(0);

    // Write cash inventory
    if let Some(cash_inventory) = inventory.get(&(InventoryType::Cash as i32)) {
        for item in cash_inventory.iter() {
            write_item(packet, item);
        }
    }
}

/// Writes an equip's data to a packet
fn write_equip(packet: &mut Packet, equip: &equip::Data) {
    let mut pos = equip.position.abs();

    if pos > 100 {
        pos -= 100;
    }

    packet.write_short(pos as i16);
    packet.write_byte(1); // item type (equip)
    packet.write_int(equip.item_id);
    packet.write_byte(0); // TODO is cash
                          // TODO if is cash write id again?
    packet.write_long(-1); // TODO equip expiration time if not permanent
    packet.write_byte(equip.upgrade_slots as u8);
    packet.write_byte(equip.level as u8);
    packet.write_short(equip.str as i16);
    packet.write_short(equip.dex as i16);
    packet.write_short(equip.int as i16);
    packet.write_short(equip.luk as i16);
    packet.write_short(equip.hp as i16);
    packet.write_short(equip.mp as i16);
    packet.write_short(equip.w_atk as i16);
    packet.write_short(equip.m_atk as i16);
    packet.write_short(equip.w_def as i16);
    packet.write_short(equip.m_def as i16);
    packet.write_short(equip.acc as i16);
    packet.write_short(equip.avoid as i16);
    packet.write_short(equip.hands as i16);
    packet.write_short(equip.speed as i16);
    packet.write_short(equip.jump as i16);
    packet.write_string(&equip.owner);
    packet.write_short(equip.flag as i16);
    // TODO if iscash write 10 0x40 bytes? and dont wirte item level stuff

    packet.write_byte(0);
    packet.write_byte(equip.item_level as u8);
    packet.write_int(0); // TODO exp nibble?
    packet.write_int(equip.vicious);
    packet.write_long(0);

    // UTC zero-timestamp
    packet.write_long(94354848000000000);
    packet.write_int(-1);
}

/// Writes an item's data to a packet
fn write_item(packet: &mut Packet, item: &item::Data) {
    // Positions are 0-indexed in db, client expects 1-indexed
    packet.write_byte((item.position + 1) as u8);
    packet.write_byte(2); // item type (item)
    packet.write_int(item.item_id);
    packet.write_byte(0); // TODO is cash
                          // TODO if is cash write id again?
    packet.write_long(-1); // TODO item expiration time if not permanent
    packet.write_short(item.amount as i16);
    packet.write_string(&item.owner);
    packet.write_short(item.flag as i16);
    // TODO if item is rechargable, sent int(2), bytes (0x54, 0, 0, 0x34)?
}

/// Writes a character's skills to a packet
fn write_character_skills(packet: &mut Packet, character: &character::Data) {
    packet.write_byte(0);
    // TODO skip hidden (aran) skills in len

    // Write skills
    if let Some(skills) = character.skills.as_ref() {
        packet.write_short(skills.len() as i16);

        for skill in skills.iter() {
            // TODO if hidden, continue
            packet.write_int(skill.skill_id);
            packet.write_int(skill.level);
            packet.write_long(-1); // TODO expiration?
                                   // TODO if skill is fourth job write int(masterlevel)
        }
    } else {
        packet.write_short(0);
    }

    // Write cooldowns
    if let Some(cooldowns) = character.cooldowns.as_ref() {
        packet.write_short(cooldowns.len() as i16);

        for cooldown in cooldowns.iter() {
            packet.write_int(cooldown.skill_id);

            let remaining = cooldown.start + cooldown.length - Utc::now().timestamp_millis();
            packet.write_short((remaining / 1000) as i16);
        }
    } else {
        packet.write_short(0);
    }
}

/// Writes a character's quests to a packet
fn write_character_quests(packet: &mut Packet, character: &character::Data) {
    let mut quests: HashMap<i32, Vec<&quest::Data>> = HashMap::new();

    for quest in character.quests.iter().flatten() {
        if !quests.contains_key(&(quest.status as i32)) {
            quests.insert(quest.status as i32, Vec::new());
        }

        quests.get_mut(&(quest.status as i32)).unwrap().push(quest);
    }

    if let Some(started_quests) = quests.get(&(QuestStatus::Started as i32)) {
        packet.write_short(started_quests.len() as i16);

        for quest in started_quests.iter() {
            packet.write_short(quest.quest_id as i16);
            packet.write_string(""); // TODO quest progress data
                                     // TODO quest info number stuff?
        }
    } else {
        packet.write_short(0);
    }

    if let Some(completed_quests) = quests.get(&(QuestStatus::Completed as i32)) {
        packet.write_short(completed_quests.len() as i16);

        for quest in completed_quests.iter() {
            packet.write_short(quest.quest_id as i16);
            packet.write_long(quest.completed as i64); // TODO this needs to be normalized
        }
    } else {
        packet.write_short(0);
    }
}

/// Writes a character's rings to a packet
fn write_character_rings(packet: &mut Packet, character: &character::Data) {
    packet.write_short(0); // TODO crush rings size
                           // TODO write each crush ring
    packet.write_short(0); // TODO friendship rings
                           // TODO write each friendship ring
    packet.write_short(0); // TODO write marriage ring if married
}

/// Writes a character's teleport rock maps to a packet
fn write_character_teleport_rock_maps(packet: &mut Packet, character: &character::Data) {
    // TODO
    // Teleport rock maps
    for i in 0..5 {
        packet.write_int(999999999);
    }

    // VIP teleport rock maps
    for i in 0..10 {
        packet.write_int(999999999);
    }
}

/// Writes a character's monster book data to a packet
fn write_character_monster_book(packet: &mut Packet, character: &character::Data) {
    packet.write_int(0); // TODO monster book cover
    packet.write_byte(0);
    packet.write_short(0); // TODO monster book cards size
                           // TODO write each card
}

/// Packet containing the character's keymap configuration
fn character_keymap(character: &character::Data) -> Packet {
    let mut packet = Packet::new();
    packet.write_short(0x14F);
    packet.write_byte(0);

    let keymaps = character.keymaps.as_ref().unwrap();
    let mut map: HashMap<i32, (u8, i32)> = HashMap::new();

    for keymap in keymaps.iter() {
        map.insert(keymap.key, (keymap.type_ as u8, keymap.action));
    }

    for key in 0..90 {
        match map.get(&key) {
            Some(binding) => {
                packet.write_byte(binding.0);
                packet.write_int(binding.1);
            }
            None => {
                packet.write_byte(0);
                packet.write_int(0);
            }
        };
    }

    packet
}

///
fn spawn_character(character: &Character, entering: bool) -> Packet {
    let mut packet = Packet::new();
    packet.write_short(0xA0);

    packet.write_int(character.data.id);
    packet.write_byte(character.data.level as u8);
    packet.write_string(&character.data.name);

    match character.data.guild {
        Some(guild_id) => {
            // TODO load guild data by id, if found write guild data
            packet.write_string("");
            packet.write_bytes(&[0, 0, 0, 0, 0, 0]);
        }
        None => {
            packet.write_string("");
            packet.write_bytes(&[0, 0, 0, 0, 0, 0]);
        }
    };

    write_buffs(&mut packet);
    // TODO need to get the correct job id based on the job, create an enum that maps all jobs to job ids? (see Job class)
    packet.write_short(0); // FIXME job id
    packets::write_character_style(&mut packet, &character.data);
    packets::write_character_equipment(&mut packet, &character.data);
    packet.write_int(0); // TODO # of heart shaped chocolate in cash inv??? why
    packet.write_int(0); // TODO item effect
    packet.write_int(0); // TODO chair id

    // Check if character is already present in the map
    if entering {
        // TODO should be set to portal closest to map spawn point
        packet.write_position((0, 0 - 42));
        packet.write_byte(6);
    } else {
        packet.write_position(character.position);
        packet.write_byte(character.stance as u8);
    }

    packet.write_short(0);
    packet.write_byte(0);

    // TODO pet info
    for i in 0..3 {
        // TODO write pet[i]
    }

    packet.write_byte(0);

    // TODO mount info
    packet.write_int(1);
    packet.write_long(0);

    // TODO shop and minigame info
    packet.write_byte(0);

    // TODO chalkboard
    packet.write_byte(0);

    // TODO crush ring
    packet.write_byte(0);

    // TODO friendship ring
    packet.write_byte(0);

    // TODO marriage ring
    packet.write_byte(0);

    // TODO new years card info
    packet.write_byte(0);

    packet.write_byte(0);
    packet.write_byte(0);
    packet.write_byte(0); // TODO team
    packet
}

///
fn write_buffs(packet: &mut Packet) {
    packet.write_int(0);
    packet.write_short(0);
    packet.write_byte(0xFC);
    packet.write_byte(1);
    packet.write_int(0); // TODO morph

    let buff_mask = 0i64;
    // TODO compute buff mask
    packet.write_int(((buff_mask >> 32) & 0xffffffffi64) as i32);
    // TODO buff value
    packet.write_int((buff_mask & 0xffffffffi64) as i32);

    // TODO energy
    packet.write_int(0);
    packet.write_short(0);
    packet.write_bytes(&[0u8; 4]);

    // TODO dash buff
    packet.write_int(0);
    packet.write_bytes(&[0u8; 11]);
    packet.write_short(0);

    // TODO dash jump
    packet.write_bytes(&[0u8; 9]);
    packet.write_int(0);
    packet.write_short(0);
    packet.write_byte(0);

    // TODO monster riding
    packet.write_long(0);

    let char_magic_spawn = random::<i32>();
    packet.write_int(char_magic_spawn);

    // Speed Infusion
    packet.write_bytes(&[0u8; 8]);
    packet.write_int(char_magic_spawn);
    packet.write_byte(0);
    packet.write_int(char_magic_spawn);
    packet.write_short(0);

    // Homing Beacon
    packet.write_bytes(&[0u8; 9]);
    packet.write_int(char_magic_spawn);
    packet.write_int(0);

    // Zombify
    packet.write_bytes(&[0u8; 9]);
    packet.write_int(char_magic_spawn);
    packet.write_short(0);
    packet.write_short(0);
}
