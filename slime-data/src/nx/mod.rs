use once_cell::sync::Lazy;
use std::{collections::HashMap, path::Path};

pub mod equipment;
pub mod map;

pub use self::equipment::Equipment;
pub use self::map::Map;

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

pub static DATA: Lazy<HashMap<&str, ::nx::File>> = Lazy::new(|| {
    let mut map = HashMap::new();

    for nx_file in NX_FILES {
        // FIXME this path is problematic, depends on where the binary is located
        let filename = format!("slime-data/nx/{}.nx", nx_file);
        let path = Path::new(&filename);

        // FIXME get rid of this unsafe, may want to move off of nx crate
        let file = unsafe { ::nx::File::open(path).unwrap() };

        map.insert(nx_file, file);
    }

    map
});
