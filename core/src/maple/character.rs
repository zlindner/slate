use super::{skill::Cooldown, Item, Pet, Skill};
use crate::pg::PgCharacter;
use std::collections::HashMap;

#[derive(Default)]
pub struct Character {
    pub channel_id: i16,
    pub pos: (i32, i32),
    pub pg: PgCharacter,
    pub pets: Vec<Pet>,
    pub skills: Vec<Skill>,
    pub cooldowns: Vec<Cooldown>,
    pub equip_inventory: HashMap<i16, Item>,
    pub use_inventory: HashMap<i16, Item>,
    pub setup_inventory: HashMap<i16, Item>,
    pub etc_inventory: HashMap<i16, Item>,
    pub cash_inventory: HashMap<i16, Item>,
}

impl Character {
    pub fn new() -> Self {
        Self::default()
    }
}
