use crate::session::ChannelSession;
use rand::random;
use slate_data::{
    maple::{
        self,
        map::{MapBroadcast, PacketBroadcast},
    },
    nx, packet,
    sql::{self, account::LoginState, item::InventoryType, quest::QuestStatus},
};
use slate_net::Packet;
use sqlx::types::chrono::{Local, Utc};
use std::{collections::HashMap, time::Duration};
use tokio::{
    sync::{broadcast, mpsc},
    time::timeout,
};

/// Channel server: connect packet (0x14)
/// Called when the client transitions from login to channel server
pub async fn handle(mut packet: Packet, session: &mut ChannelSession) -> anyhow::Result<()> {
    let session_id = packet.read_int();

    let login_session = match sql::LoginSession::load_optional(session_id, &session.db).await? {
        Some(login_session) => login_session,
        None => {
            return session
                .stream
                .write_packet(connect_error(ConnectError::Unknown))
                .await;
        }
    };

    // Delete the login session from db
    login_session.delete(&session.db).await?;

    let account =
        match sql::Account::load_optional_by_id(login_session.account_id, &session.db).await? {
            Some(account) => account,
            None => {
                return session
                    .stream
                    .write_packet(connect_error(ConnectError::Unknown))
                    .await;
            }
        };

    session.account_id = Some(account.id);

    // Ensure that account has the `Transitioning` state
    if !matches!(account.state, LoginState::Transitioning) {
        return session
            .stream
            .write_packet(connect_error(ConnectError::Unknown))
            .await;
    }

    // Set the account's state to `LoggedIn`
    sql::Account::update_login_state(account.id, LoginState::LoggedIn, &session.db).await?;

    let character = maple::Character::load(login_session.character_id, &session.db).await?;

    session
        .stream
        .write_packet(character_info(login_session.channel_id, &character))
        .await?;

    session
        .stream
        .write_packet(character_keymap(&character))
        .await?;

    let map = maple::Map::load(character.data.map).unwrap();

    let broadcast_tx = session.state.get_map_broadcast_tx(map.id).clone();

    let (tx, mut rx) = mpsc::channel(64);
    let joined_broadcast = MapBroadcast::Joined(tx);

    // Notify other characters that we joined the map, and get the number of characters we notified
    // NOTE: it is possible that we send a broadcast to a character who disconnects before responding --
    // this is handled by adding a timeout to the below rx.recv call
    let other_characters = broadcast_tx.send(joined_broadcast)? - 1;

    log::debug!("Start receiving {} map characters...", other_characters);

    // Listen for responses from the map's characters
    // TODO should listen for spawn_character packet, is probably less bytes
    for i in 0..other_characters {
        log::debug!("Receiving character {}", i);

        // TODO tweak timeout
        let character = match timeout(Duration::from_secs(1), rx.recv()).await {
            Ok(Some(character)) => character,
            _ => {
                log::warn!("Didn't receive character {} in time", i);
                continue;
            }
        };

        log::debug!("Received character {}!", i);

        session
            .stream
            .write_packet(spawn_character(&character, false))
            .await?;
    }

    // Send the map's npcs
    for npc in map.data.npcs.values() {
        session.stream.write_packet(spawn_npc(npc)).await?;
        session
            .stream
            .write_packet(spawn_npc_request_controller(npc))
            .await?;
    }

    // Send the map's portals
    for portal in map.data.portals.values() {
        session
            .stream
            .write_packet(spawn_portal(map.id, portal))
            .await?;
    }

    let broadcast = MapBroadcast::Packet(PacketBroadcast {
        packet: spawn_character(&character, true),
        sender_id: character.data.id,
        send_to_sender: false,
    });
    broadcast_tx.send(broadcast)?;

    // Move the character into the current session
    session.character = Some(character);

    // Subscribe to the current map's broadcast channel
    session.map_broadcast_tx = Some(broadcast_tx.clone());
    session.map_broadcast_rx = Some(broadcast_tx.subscribe());

    Ok(())
}

/// Possible connect errors
enum ConnectError {
    Unknown = 0x09,
}

/// Packet containing the reason for failing to connect
fn connect_error(error: ConnectError) -> Packet {
    let mut packet = Packet::new(0x09);
    packet.write_short(error as i16);
    packet
}

///
fn character_info(channel_id: i32, character: &maple::Character) -> Packet {
    let mut packet = Packet::new(0x7D);
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
fn write_character(packet: &mut Packet, character: &maple::Character) {
    packet.write_long(-1);
    packet.write_byte(0);
    packet::write_character_stats(packet, &character.data);
    packet.write_byte(character.data.buddy_capacity as u8);

    // TODO blessing of the fairy stuff
    packet.write_byte(0);

    packet.write_int(character.data.mesos);
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
fn write_character_inventory(packet: &mut Packet, character: &maple::Character) {
    packet.write_byte(character.data.equip_slots as u8);
    packet.write_byte(character.data.use_slots as u8);
    packet.write_byte(character.data.setup_slots as u8);
    packet.write_byte(character.data.etc_slots as u8);
    packet.write_byte(character.data.cash_slots as u8);

    // UTC zero-timestamp
    packet.write_long(94354848000000000);

    // Write equips
    for equip in character.equipment.iter() {
        write_equip(packet, equip);
    }

    packet.write_short(0);
    // TODO write equipped cash items
    packet.write_short(0);

    let mut inventory: HashMap<i32, Vec<&sql::Item>> = HashMap::new();

    for item in character.items.iter() {
        inventory.entry(item.inventory_type as i32).or_default();
        inventory
            .get_mut(&(item.inventory_type as i32))
            .unwrap()
            .push(item);
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
fn write_equip(packet: &mut Packet, equip: &sql::Equipment) {
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
fn write_item(packet: &mut Packet, item: &sql::Item) {
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
fn write_character_skills(packet: &mut Packet, character: &maple::Character) {
    packet.write_byte(0);
    // TODO skip hidden (aran) skills in len

    // Write skills
    if !character.skills.is_empty() {
        packet.write_short(character.skills.len() as i16);

        for skill in character.skills.iter() {
            // TODO if hidden, continue
            packet.write_int(skill.id);
            packet.write_int(skill.level);
            packet.write_long(-1); // TODO expiration?
                                   // TODO if skill is fourth job write int(masterlevel)
        }
    } else {
        packet.write_short(0);
    }

    // Write cooldowns
    if !character.cooldowns.is_empty() {
        packet.write_short(character.cooldowns.len() as i16);

        for cooldown in character.cooldowns.iter() {
            packet.write_int(cooldown.skill_id);

            let remaining = cooldown.start + cooldown.length - Utc::now().timestamp_millis();
            packet.write_short((remaining / 1000) as i16);
        }
    } else {
        packet.write_short(0);
    }
}

/// Writes a character's quests to a packet
fn write_character_quests(packet: &mut Packet, character: &maple::Character) {
    let mut quests: HashMap<i32, Vec<&sql::Quest>> = HashMap::new();

    for quest in character.quests.iter() {
        quests.entry(quest.status as i32).or_default();
        quests.get_mut(&(quest.status as i32)).unwrap().push(quest);
    }

    if let Some(started_quests) = quests.get(&(QuestStatus::Started as i32)) {
        packet.write_short(started_quests.len() as i16);

        for quest in started_quests.iter() {
            packet.write_short(quest.id as i16);
            packet.write_string(""); // TODO quest progress data
                                     // TODO quest info number stuff?
        }
    } else {
        packet.write_short(0);
    }

    if let Some(completed_quests) = quests.get(&(QuestStatus::Completed as i32)) {
        packet.write_short(completed_quests.len() as i16);

        for quest in completed_quests.iter() {
            packet.write_short(quest.id as i16);
            packet.write_long(quest.completed as i64); // TODO this needs to be normalized
        }
    } else {
        packet.write_short(0);
    }
}

/// Writes a character's rings to a packet
fn write_character_rings(packet: &mut Packet, character: &maple::Character) {
    packet.write_short(0); // TODO crush rings size
                           // TODO write each crush ring
    packet.write_short(0); // TODO friendship rings
                           // TODO write each friendship ring
    packet.write_short(0); // TODO write marriage ring if married
}

/// Writes a character's teleport rock maps to a packet
fn write_character_teleport_rock_maps(packet: &mut Packet, character: &maple::Character) {
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
fn write_character_monster_book(packet: &mut Packet, character: &maple::Character) {
    packet.write_int(0); // TODO monster book cover
    packet.write_byte(0);
    packet.write_short(0); // TODO monster book cards size
                           // TODO write each card
}

/// Packet containing the character's keymap configuration
fn character_keymap(character: &maple::Character) -> Packet {
    let mut packet = Packet::new(0x14F);
    packet.write_byte(0);

    let mut map: HashMap<i32, (u8, i32)> = HashMap::new();

    for keymap in character.keymaps.iter() {
        map.insert(keymap.key_id, (keymap.key_type as u8, keymap.action));
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
fn spawn_character(character: &maple::Character, entering: bool) -> Packet {
    let mut packet = Packet::new(0xA0);
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
    packet::write_character_style(&mut packet, &character.data);
    packet::write_character_equipment(&mut packet, &character.equipment);
    packet.write_int(0); // TODO # of heart shaped chocolate in cash inv??? why
    packet.write_int(0); // TODO item effect
    packet.write_int(0); // TODO chair id

    // Check if character is already present in the map
    if entering {
        // TODO should be set to portal closest to map spawn point
        packet.write_position((0, 0 - 42));
        packet.write_byte(6);
    } else {
        packet.write_position(character.pos);
        packet.write_byte(character.stance);
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

///
fn spawn_npc(npc: &nx::map::Life) -> Packet {
    let mut packet = Packet::new(0x101);
    packet.write_int(npc.object_id);
    packet.write_int(npc.id);
    packet.write_short(npc.position.0);
    packet.write_short(npc.cy);
    packet.write_byte((npc.f != 1) as u8);
    packet.write_short(npc.fh);
    packet.write_short(npc.rx0);
    packet.write_short(npc.rx1);
    packet.write_byte(1);
    packet
}

///
fn spawn_npc_request_controller(npc: &nx::map::Life) -> Packet {
    let mut packet = Packet::new(0x103);
    packet.write_byte(1);
    packet.write_int(npc.object_id);
    packet.write_int(npc.id);
    packet.write_short(npc.position.0);
    packet.write_short(npc.cy);
    packet.write_byte((npc.f != 1) as u8);
    packet.write_short(npc.fh);
    packet.write_short(npc.rx0);
    packet.write_short(npc.rx1);
    packet.write_byte(1);
    packet
}

// TODO DoorObject.sendSpawnData
fn spawn_portal(map_id: i32, portal: &nx::map::Portal) -> Packet {
    let mut packet = Packet::new(0x43);
    packet.write_int(map_id);
    packet.write_int(portal.target_map_id as i32);
    packet.write_short(portal.x as i16);
    packet.write_short(portal.y as i16);
    packet
}
