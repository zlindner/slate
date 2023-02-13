use crate::map::Map;
use dashmap::{mapref::one::Ref, DashMap};
use oxy_core::{nx, prisma::PrismaClient};
use std::{collections::HashMap, sync::Arc};

pub struct Shared {
    pub db: Arc<PrismaClient>,
    pub maps: HashMap<i32, Map>,
    quests: DashMap<i32, nx::Quest>,
}

impl Shared {
    pub fn new(db: Arc<PrismaClient>) -> Self {
        Self {
            db,
            //maps: nx::load_maps(), // TODO merge Map into one type rename nx one to MapData
            maps: HashMap::new(),
            quests: DashMap::new(),
        }
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
