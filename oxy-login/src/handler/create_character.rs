use super::character_list::write_character;
use anyhow::Result;
use once_cell::sync::Lazy;
use oxy_core::{
    net::{Client, Packet},
    nx::{self, EquipCategory},
    prisma::{character, keymap, InventoryType},
};
use std::collections::HashSet;

/// Login server: create character packet (0x16)
///
pub async fn handle(mut packet: Packet, client: &mut Client) -> Result<()> {
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
        log::error!(
            "Client tried to packet edit in character creation (account id {})",
            client.session.account_id
        );
        client.disconnect().await;
        return Ok(());
    }

    // Check if the client has a free character slot in the current world
    let world_characters = client
        .db
        .character()
        .find_many(vec![
            character::account_id::equals(client.session.account_id),
            character::world_id::equals(client.session.world_id),
        ])
        .exec()
        .await?;

    if world_characters.len() >= 3 {
        return Ok(());
    }

    let (starter_item, job_id, map) = match job {
        0 => (4161047, 1000, 130030000), // Knight of Cygnus (noblesse guide, noblesse, noblesse starting map)
        1 => (4161001, 0, 10000),        // Beginner (beginner's guide, explorer, mushroom town)
        2 => (4161048, 2000, 914000000), // Aran (legend's guide, legend, aran tutorial start)
        _ => {
            log::error!("Invalid/unsupported job: {}", job);
            // TODO
            return Ok(());
        }
    };

    // Create character
    let mut character = client
        .db
        .character()
        .create(
            client.session.account_id,
            client.session.world_id,
            name,
            job_id,
            skin_colour,
            gender as i32,
            hair + hair_colour,
            face,
            map,
            vec![],
        )
        .exec()
        .await?;

    // Create equips
    let top_equip = client
        .db
        .equip()
        .create(
            top,
            character::id::equals(character.id),
            5,
            nx::get_equip_data(top, EquipCategory::Top),
        )
        .exec()
        .await?;

    let bottom_equip = client
        .db
        .equip()
        .create(
            bottom,
            character::id::equals(character.id),
            6,
            nx::get_equip_data(bottom, EquipCategory::Bottom),
        )
        .exec()
        .await?;

    let shoe_equip = client
        .db
        .equip()
        .create(
            shoes,
            character::id::equals(character.id),
            7,
            nx::get_equip_data(shoes, EquipCategory::Shoes),
        )
        .exec()
        .await?;

    let weapon_equip = client
        .db
        .equip()
        .create(
            weapon,
            character::id::equals(character.id),
            11,
            nx::get_equip_data(weapon, EquipCategory::Weapon),
        )
        .exec()
        .await?;

    // Create default keymap
    let keymap_creates = (0..40).map(|i| {
        client.db.keymap().create(
            character::id::equals(character.id),
            DEFAULT_KEYS[i],
            DEFAULT_TYPES[i],
            DEFAULT_ACTIONS[i],
            vec![],
        )
    });

    let keymaps: Vec<keymap::Data> = client.db._batch(keymap_creates).await?;

    // Add starter item to etc inventory
    let item = client
        .db
        .item()
        .create(
            starter_item,
            character::id::equals(character.id),
            InventoryType::Etc,
            0,
            1,
            vec![],
        )
        .exec()
        .await?;

    // This doesn't set/update any db data, just for convenience when calling create_character
    character.equips = Some(vec![top_equip, bottom_equip, shoe_equip, weapon_equip]);
    character.keymaps = Some(keymaps);
    character.items = Some(vec![item]);

    let response = create_character(character);
    client.send(response).await?;
    Ok(())
}

///
pub fn create_character(character: character::Data) -> Packet {
    let mut packet = Packet::new();
    packet.write_short(0x0E);
    packet.write_byte(0);
    write_character(&mut packet, &character, false);
    packet
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
