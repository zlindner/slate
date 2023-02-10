use crate::map::Map;
use dashmap::{mapref::one::Ref, DashMap};
use oxy_core::{nx, prisma::PrismaClient};
use std::sync::Arc;

pub struct Shared {
    pub db: Arc<PrismaClient>,
    maps: DashMap<i32, Map>,
    quests: DashMap<i32, nx::Quest>,
}

impl Shared {
    pub fn new(db: Arc<PrismaClient>) -> Self {
        Self {
            db,
            maps: DashMap::new(),
            quests: DashMap::new(),
        }
    }

    /// Gets a map by id, loading it from Map.nx if not yet loaded
    pub fn get_map(&self, map_id: i32) -> Ref<'_, i32, Map> {
        if map_id < 0 {
            log::debug!("Tried to get invalid map: {}", map_id);
        }

        if !self.maps.contains_key(&map_id) {
            self.maps.insert(map_id, Map::new(map_id));
        }

        self.maps.get(&map_id).unwrap()
    }

    /// Checks if the given map is loaded
    pub fn is_map_loaded(&self, map_id: i32) -> bool {
        if map_id == -1 {
            return false;
        }

        self.maps.contains_key(&map_id)
    }

    /// Gets a quest by id, loading it from Quest.nx if not yet loaded
    pub fn get_quest(&self, quest_id: i32) -> Ref<'_, i32, nx::Quest> {
        if !self.quests.contains_key(&quest_id) {
            let quest = nx::quest::load_quest(quest_id);

            if quest.is_some() {
                self.quests.insert(quest_id, quest.unwrap());
            }
        }

        self.quests.get(&quest_id).unwrap()
    }
}
