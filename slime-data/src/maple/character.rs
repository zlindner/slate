use crate::{sql, Db};

pub struct Character {
    pub data: sql::Character,
    pub equipment: Vec<sql::Equipment>, // TODO might want to make a map?
    pub items: Vec<sql::Item>,
}

impl Character {
    pub async fn load(name: &String, world_id: i32, db: &Db) -> anyhow::Result<Self> {
        let character = sql::Character::load_by_name(name, world_id, db).await?;
        let equipment = sql::Equipment::load_all(character.id, db).await?;
        let items = sql::Item::load_all(character.id, db).await?;

        Ok(Self {
            data: character,
            equipment,
            items,
        })
    }
}
