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
        if !self.maps.contains_key(&map_id) {
            self.maps.insert(map_id, Map::new(map_id));
        }

        self.maps.get(&map_id).unwrap()
    }
}
