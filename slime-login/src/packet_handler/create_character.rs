use super::character_list::write_character;
use crate::server::LoginSession;
use once_cell::sync::Lazy;
use slime_data::{
    nx::{self, equipment::EquipmentType},
    sql::{self, item::InventoryType},
};
use slime_net::Packet;
use sqlx::{MySql, QueryBuilder};
use std::collections::HashSet;

/// Login server: create character packet (0x16)
/// Handles new character creation
pub async fn handle(mut packet: Packet, session: &mut LoginSession) -> anyhow::Result<()> {
    let name = packet.read_string();
    let job = packet.read_int();
    let face = packet.read_int();
    let hair = packet.read_int();
    let hair_colour = packet.read_int();
    let skin_colour = packet.read_int();
    let top = packet.read_int();
    let bottom = packet.read_int();
    let shoes = packet.read_int();
    let weapon = packet.read_int();
    let gender = packet.read_byte();

    // Character has invalid equipment (via packet editing)
    if !STARTER_WEAPONS.contains(&weapon)
        || !STARTER_TOPS.contains(&top)
        || !STARTER_BOTTOMS.contains(&bottom)
        || !STARTER_SHOES.contains(&shoes)
        || !STARTER_HAIR.contains(&hair)
        || !STARTER_FACE.contains(&face)
    {
        log::warn!(
            "Client tried to packet edit in character creation (account id {})",
            session.data.account_id
        );
        return session.stream.close().await;
    }

    // Get number of characters the account currently has in the world
    let num_characters =
        sql::Character::get_count(session.data.account_id, session.data.world_id, &session.db)
            .await?;

    if num_characters >= 3 {
        log::debug!("Player already has 3 characters in the selected world");
        return Ok(());
    }

    let (starter_item_id, job_id, map) = match job {
        0 => (4161047, 1000, 130030000), // Knight of Cygnus (noblesse guide, noblesse, noblesse starting map)
        1 => (4161001, 0, 10000),        // Beginner (beginner's guide, explorer, mushroom town)
        2 => (4161048, 2000, 914000000), // Aran (legend's guide, legend, aran tutorial start)
        _ => {
            log::error!("Invalid/unsupported job: {}", job);
            return Ok(());
        }
    };

    session.data.map_id = map;

    // Create the character and get it's id
    let character_id = sqlx::query(
        "INSERT INTO characters (account_id, world_id, name, job, skin_colour, gender, hair, face, map)
        VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(session.data.account_id)
    .bind(session.data.world_id)
    .bind(&name)
    .bind(job_id)
    .bind(skin_colour)
    .bind(gender as i32)
    .bind(hair + hair_colour)
    .bind(face)
    .bind(session.data.map_id)
    .execute(&session.db)
    .await?
    .last_insert_id() as i32;

    // Create starter equips
    let starter_equips = [
        (top, EquipmentType::Top),
        (bottom, EquipmentType::Bottom),
        (shoes, EquipmentType::Shoes),
        (weapon, EquipmentType::Weapon),
    ];
    create_equips(starter_equips, character_id, session).await?;

    create_default_keymaps(session).await?;

    // Create starter item
    sqlx::query(
        "INSERT INTO items (item_id, character_id, inventory_type, position, amount)
        VALUES (?, ?, ?, ?, ?)",
    )
    .bind(starter_item_id)
    .bind(character_id)
    .bind(InventoryType::Etc)
    .bind(0)
    .bind(1)
    .execute(&session.db)
    .await?;

    let character = sql::Character::load(character_id, &session.db).await?;
    let equipment = sql::Equipment::load_all(character_id, &session.db).await?;

    session
        .stream
        .write_packet(create_character(&character, &equipment))
        .await?;

    Ok(())
}

///
pub fn create_character(character: &sql::Character, equipment: &[sql::Equipment]) -> Packet {
    let mut packet = Packet::new(0x0E);
    packet.write_byte(0);
    write_character(&mut packet, character, equipment, false);
    packet
}

async fn create_equips(
    equips: [(i32, EquipmentType); 4],
    character_id: i32,
    session: &mut LoginSession,
) -> anyhow::Result<()> {
    let mut query_builder = QueryBuilder::<MySql>::new(
        "INSERT INTO equipment (item_id, character_id, position, w_atk, upgrade_slots) ",
    );

    query_builder.push_values(equips, |mut builder, (id, equip_type)| {
        let nx_equip = nx::Equipment::load(id, &equip_type).unwrap();

        builder
            .push_bind(id)
            .push_bind(character_id)
            .push_bind(equip_type.get_position())
            .push_bind(nx_equip.w_atk.unwrap_or(0))
            .push_bind(nx_equip.upgrade_slots.unwrap_or(0));
    });

    query_builder.build().execute(&session.db).await?;
    Ok(())
}

async fn create_default_keymaps(session: &mut LoginSession) -> anyhow::Result<()> {
    let keymaps = DEFAULT_KEYS
        .iter()
        .zip(DEFAULT_TYPES.iter())
        .zip(DEFAULT_ACTIONS.iter())
        .map(|((x, y), z)| (x, y, z));

    let mut query_builder =
        QueryBuilder::<MySql>::new("INSERT INTO keymaps (character_id, key_id, key_type, action) ");

    query_builder.push_values(keymaps, |mut builder, (key, type_, action)| {
        builder
            .push_bind(session.data.character_id)
            .push_bind(key)
            .push_bind(type_)
            .push_bind(action);
    });

    query_builder.build().execute(&session.db).await?;
    Ok(())
}

static STARTER_WEAPONS: Lazy<HashSet<i32>> = Lazy::new(|| {
    [
        1302000, // sword
        1312004, // hand axe
        1322005, // wooden club
        1442079, // basic polearm
    ]
    .into_iter()
    .collect()
});

static STARTER_TOPS: Lazy<HashSet<i32>> = Lazy::new(|| {
    [
        1040002, // white undershirt
        1040006, // undershirt
        1040010, // grey t-shirt
        1041002, // white tubetop
        1041006, // yellow t-shirt
        1041010, // green t-shirt
        1041011, // red striped top
        1042167, // simple warrior top
    ]
    .into_iter()
    .collect()
});

static STARTER_BOTTOMS: Lazy<HashSet<i32>> = Lazy::new(|| {
    [
        1060002, // blue jean shorts
        1060006, // brown cotton shorts
        1061002, // red miniskirt
        1061008, // indigo miniskirt
        1062115, // simple warrior pants
    ]
    .into_iter()
    .collect()
});

static STARTER_SHOES: Lazy<HashSet<i32>> = Lazy::new(|| {
    [
        1072001, // red rubber boots
        1072005, // leather sandals
        1072037, // yellow rubber boots
        1072038, // blue rubber boots
        1072383, // average musashi shoes
    ]
    .into_iter()
    .collect()
});

static STARTER_HAIR: Lazy<HashSet<i32>> = Lazy::new(|| {
    [
        30000, // toben
        30010, // zeta
        30020, // rebel
        30030, // buzz
        31000, // sammy
        31040, // edgie
        31050, // connie
    ]
    .into_iter()
    .collect()
});

static STARTER_FACE: Lazy<HashSet<i32>> = Lazy::new(|| {
    [
        20000, // motivated look (m)
        20001, // perplexed stare
        20002, // leisure look (m)
        21000, // motiviated look (f)
        21001, // fearful stare (m)
        21002, // leisure look (f)
        21201, // fearful stare (f)
        20401, // perplexed stare hazel
        20402, // leisure look hazel
        21700, // motivated look amethyst
        20100, // motivated look blue
    ]
    .into_iter()
    .collect()
});

const DEFAULT_KEYS: [i32; 40] = [
    18, 65, 2, 23, 3, 4, 5, 6, 16, 17, 19, 25, 26, 27, 31, 34, 35, 37, 38, 40, 43, 44, 45, 46, 50,
    56, 59, 60, 61, 62, 63, 64, 57, 48, 29, 7, 24, 33, 41, 39,
];

const DEFAULT_TYPES: [i32; 40] = [
    4, 6, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 5, 5, 4, 4, 5, 6, 6, 6, 6, 6, 6,
    5, 4, 5, 4, 4, 4, 4, 4,
];

const DEFAULT_ACTIONS: [i32; 40] = [
    0, 106, 10, 1, 12, 13, 18, 24, 8, 5, 4, 19, 14, 15, 2, 17, 11, 3, 20, 16, 9, 50, 51, 6, 7, 53,
    100, 101, 102, 103, 104, 105, 54, 22, 52, 21, 25, 26, 23, 27,
];
