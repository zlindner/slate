use crate::character::Character;
use anyhow::Result;
use dashmap::DashMap;
use oxy_core::{
    net::{BroadcastPacket, Packet},
    nx,
};
use std::collections::HashMap;
use tokio::sync::broadcast::{self, Sender};

pub struct Map {
    pub id: i32,
    pub characters: DashMap<i32, Character>,
    pub npcs: HashMap<i32, nx::Life>,
    pub monsters: HashMap<i32, nx::Life>,
    pub broadcast_tx: Sender<BroadcastPacket>,
}

impl Map {
    pub fn new(id: i32) -> Self {
        // TODO error handle
        let map_data = nx::load_map(id).unwrap();
        let (tx, _rx) = broadcast::channel::<BroadcastPacket>(100);

        Self {
            id,
            characters: DashMap::new(),
            npcs: map_data.npcs,
            monsters: map_data.monsters,
            broadcast_tx: tx,
        }
    }

    /// Broadcasts the packet to all characters in the current map.
    pub fn broadcast(
        &self,
        packet: Packet,
        sender: &Character,
        send_to_sender: bool,
    ) -> Result<()> {
        let broadcast_packet = BroadcastPacket {
            packet,
            sender_id: sender.id,
            sender_position: sender.position,
            send_to_sender,
        };
        self.broadcast_tx.send(broadcast_packet)?;
        Ok(())
    }
}
