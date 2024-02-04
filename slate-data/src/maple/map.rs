use crate::nx;
use slate_net::Packet;
use tokio::sync::{broadcast, mpsc};

pub struct Map {
    pub id: i32,
    pub data: nx::Map,

    pub broadcast_tx: broadcast::Sender<MapBroadcast>,

    // Broadcast receiver isn't used, but we need to store it so it doesn't get
    // dropped and close the channel
    _broadcast_rx: broadcast::Receiver<MapBroadcast>,
}

impl Map {
    pub fn load(id: i32) -> anyhow::Result<Self> {
        let data = nx::Map::load(id)?;

        // TODO tweak capacity (is per map)?
        let (tx, rx) = broadcast::channel(32);

        Ok(Self {
            id,
            data,
            broadcast_tx: tx,
            _broadcast_rx: rx,
        })
    }

    /// Gets the closest spawn point to a position in the current map
    /// TODO looks like spawn points also have the name "sp" need to see if this is always the case
    pub fn get_closest_spawn_point(&self, pos: (i32, i32)) -> Option<&nx::Portal> {
        let mut closest = None;
        let mut closest_distance = std::i32::MAX;

        for (_, portal) in self.data.portals.iter() {
            // TODO make const -- MAP_NONE
            // Portal must be a spawn point -- target map id is NONE
            if portal.target_map_id != 999999999 {
                continue;
            }

            // Portal must be type 0 or 1
            if !(portal.type_ == 0 || portal.type_ == 1) {
                continue;
            }

            let distance = (pos.0 - portal.x).pow(2) + (pos.1 - portal.y).pow(2);

            if distance < closest_distance {
                closest_distance = distance;
                closest = Some(portal);
            }
        }

        closest
    }

    /// Gets a portal in the current map by name
    pub fn get_portal_by_name(&self, name: String) -> Option<&nx::Portal> {
        let portal = None;

        for (_, portal) in self.data.portals.iter() {
            if portal.name == name {
                return Some(portal);
            }
        }

        portal
    }
}

#[derive(Debug, Clone)]
pub enum MapBroadcast {
    Packet(PacketBroadcast),
    Joined(mpsc::Sender<super::Character>),
}

#[derive(Debug, Clone)]
pub struct PacketBroadcast {
    pub packet: Packet,
    pub sender_id: i32,
    pub send_to_sender: bool,
}
