use std::{collections::HashMap, path::Path};

use once_cell::sync::Lazy;

pub(crate) mod equipment;
pub use self::equipment::EquipmentType;
pub use self::equipment::NxEquipment;

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
        // FIXME this path is problematic, depends on where the binary is located
        let filename = format!("slime-nx/nx/{}.nx", nx_file);
        let path = Path::new(&filename);

        // FIXME get rid of this unsafe, may want to move off of nx crate
        let file = unsafe { nx::File::open(path).unwrap() };

        map.insert(nx_file, file);
    }

    map
});
