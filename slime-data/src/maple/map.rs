use crossbeam_channel::{Receiver, Sender};
use slime_net::Packet;
use std::collections::HashMap;

pub struct Map {
    pub broadcast_tx: Sender<Broadcast>,
    pub broadcast_rx: Receiver<Broadcast>,
    pub characters: HashMap<i32, super::Character>,
}

impl Map {
    pub fn load(id: i32) -> Self {
        let (tx, rx) = crossbeam_channel::unbounded();

        Self {
            broadcast_tx: tx,
            broadcast_rx: rx,
            characters: HashMap::new(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct Broadcast {
    pub packet: Packet,
    pub sender_id: i32,
    pub sender_position: (i32, i32),
    pub send_to_sender: bool,
}
