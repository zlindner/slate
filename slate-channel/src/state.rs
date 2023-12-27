use dashmap::DashMap;
use slate_data::maple::map::MapBroadcast;
use tokio::sync::broadcast;

pub struct State {
    map_broadcast: DashMap<
        i32,
        (
            broadcast::Sender<MapBroadcast>,
            broadcast::Receiver<MapBroadcast>,
        ),
    >,
}

impl State {
    pub fn new() -> Self {
        Self {
            map_broadcast: DashMap::new(),
        }
    }

    // TODO we can get rid of the dashmap if we just create channels for every map on init -- not sure how big memory
    // would take
    pub fn get_map_broadcast_tx(&self, map_id: i32) -> broadcast::Sender<MapBroadcast> {
        if !self.map_broadcast.contains_key(&map_id) {
            // TODO tweak channel size
            self.map_broadcast.insert(map_id, broadcast::channel(64));
        }

        self.map_broadcast.get(&map_id).unwrap().0.clone()
    }
}
