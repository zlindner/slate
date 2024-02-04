use std::collections::HashSet;

use crate::{sql, Db};

#[derive(Debug, Clone)]
pub struct Character {
    pub pos: (i32, i32),
    pub stance: u8,

    pub data: sql::Character,
    pub equipment: Vec<sql::Equipment>, // TODO might want to make a map?
    pub items: Vec<sql::Item>,
    pub keymaps: Vec<sql::Keymap>,
    pub skills: Vec<sql::Skill>,
    pub cooldowns: Vec<sql::Cooldown>,
    pub quests: Vec<sql::Quest>,

    pub blocked_portals: HashSet<String>,
}

impl Character {
    /// Loads a character by id
    pub async fn load(id: i32, db: &Db) -> anyhow::Result<Self> {
        let character = sql::Character::load(id, db).await?;
        let equipment = sql::Equipment::load_all(id, db).await?;
        let items = sql::Item::load_all(id, db).await?;
        let keymaps = sql::Keymap::load_all(id, db).await?;

        // TODO load
        let skills = Vec::new();
        let cooldowns = Vec::new();
        let quests = Vec::new();

        Ok(Self {
            pos: (0, 0),
            stance: 0,
            data: character,
            equipment,
            items,
            keymaps,
            skills,
            cooldowns,
            quests,
            blocked_portals: HashSet::new(),
        })
    }
}
