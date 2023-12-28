use crate::nx;
use slate_net::Packet;
use std::collections::HashMap;
use tokio::sync::{broadcast, mpsc};

pub struct Map {
    pub id: i32,
    pub data: nx::Map,

    pub broadcast_tx: broadcast::Sender<MapBroadcast>,

    // Broadcast receiver isn't used, but we need to store it so it doesn't get
    // dropped and close the channel
    _broadcast_rx: broadcast::Receiver<MapBroadcast>,

    pub characters: HashMap<i32, super::Character>,
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
            characters: HashMap::new(),
        })
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
