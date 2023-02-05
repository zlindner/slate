use crate::map::Map;
use dashmap::{mapref::one::Ref, DashMap};
use oxy_core::prisma::PrismaClient;
use std::sync::Arc;

pub struct Shared {
    pub db: Arc<PrismaClient>,
    maps: DashMap<i32, Map>,
}

impl Shared {
    pub fn new(db: Arc<PrismaClient>) -> Self {
        Self {
            db,
            maps: DashMap::new(),
        }
    }

    pub fn get_map(&self, map_id: i32) -> Ref<'_, i32, Map> {
        if map_id < 0 {
            log::debug!("Tried to get map: {}", map_id);
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
}
