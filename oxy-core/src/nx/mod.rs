use crate::prisma::equip;
use nx::GenericNode;
use once_cell::sync::Lazy;
use std::{collections::HashMap, path::Path};

const NX_FILES: [&str; 13] = [
    "Base",
    "Character",
    "Effect",
    "Etc",
    "Item",
    "Map",
    "Morph",
    "Npc",
    "Quest",
    "Reactor",
    "String",
    "TamingMob",
    "UI",
];

pub static DATA: Lazy<HashMap<&str, nx::File>> = Lazy::new(|| {
    let mut map = HashMap::new();

    for nx_file in NX_FILES {
        let filename = format!("oxy-core/nx/{}.nx", nx_file);
        let path = Path::new(&filename);

        // FIXME get rid of this unsafe, may want to move off of nx crate
        let file = unsafe { nx::File::open(&path).unwrap() };

        map.insert(nx_file, file);
    }

    map
});

pub enum EquipCategory {
    Bottom,
    Overall,
    Shield,
    Shoes,
    Top,
    Weapon,
}

impl EquipCategory {
    fn as_str(&self) -> &'static str {
        match self {
            Self::Bottom => "Pants",
            Self::Overall => "Longcoat",
            Self::Shield => "Shield",
            Self::Shoes => "Shoes",
            Self::Top => "Coat",
            Self::Weapon => "Weapon",
        }
    }
}

pub fn get_equip_data(equip_id: i32, equip_category: EquipCategory) -> Vec<equip::SetParam> {
    let id = format!("0{}.img", equip_id);
    let root = DATA.get("Character").unwrap().root();
    let category = root.get(equip_category.as_str()).unwrap();
    let equip = category.get(&id);

    if equip.is_none() {
        log::debug!("{} in category {} not found", id, equip_category.as_str());
        return Vec::new();
    }

    let info = equip.unwrap().get("info").unwrap();
    log::debug!("Loaded equip data from {}", id);

    let mut data = Vec::new();

    for i in info.iter() {
        match i.name() {
            "incPAD" => data.push(equip::w_atk::set(i.integer().unwrap() as i32)),
            "tuc" => data.push(equip::upgrade_slots::set(i.integer().unwrap() as i32)),
            _ => (),
        }
    }

    data
}
