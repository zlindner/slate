use crate::nx;
use slime_net::Packet;
use std::collections::HashMap;
use tokio::sync::broadcast;

pub struct Map {
    pub id: i32,
    pub data: nx::Map,

    pub broadcast_tx: broadcast::Sender<Broadcast>,

    // Broadcast receiver isn't used, but we need to store it so it doesn't get
    // dropped and close the channel
    _broadcast_rx: broadcast::Receiver<Broadcast>,

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
pub struct Broadcast {
    pub packet: Packet,
    pub sender_id: i32,
    pub sender_pos: (i32, i32),
    pub send_to_sender: bool,
}
