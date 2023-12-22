use dashmap::{
    mapref::one::{Ref, RefMut},
    DashMap,
};
use slate_data::maple;

pub struct State {
    maps: DashMap<i32, maple::Map>,
}

impl State {
    pub fn new() -> Self {
        Self {
            maps: DashMap::new(),
        }
    }

    pub fn get_map(&self, map_id: i32) -> Ref<'_, i32, maple::Map> {
        if !self.maps.contains_key(&map_id) {
            let map = maple::Map::load(map_id).unwrap();
            self.maps.insert(map_id, map);
        }

        self.maps.get(&map_id).unwrap()
    }

    pub fn get_map_mut(&self, map_id: i32) -> RefMut<'_, i32, maple::Map> {
        if !self.maps.contains_key(&map_id) {
            let map = maple::Map::load(map_id).unwrap();
            self.maps.insert(map_id, map);
        }

        self.maps.get_mut(&map_id).unwrap()
    }
}
