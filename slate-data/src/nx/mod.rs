use once_cell::sync::Lazy;
use std::{collections::HashMap, path::Path};

pub mod equipment;
pub mod map;
pub mod portal;
pub mod quest;
pub mod quest_action;
pub mod quest_requirement;

pub use self::equipment::Equipment;
pub use self::map::Map;
pub use self::portal::Portal;
pub use self::quest::Quest;
pub use self::quest_action::QuestActionType;
pub use self::quest_requirement::QuestRequirementType;

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
        let filename = format!("slate-data/nx/{}.nx", nx_file);
        let path = Path::new(&filename);

        // FIXME get rid of this unsafe, may want to move off of nx crate
        let file = unsafe { ::nx::File::open(path).unwrap() };

        map.insert(nx_file, file);
    }

    map
});
