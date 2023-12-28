use crate::Db;
use sqlx::FromRow;

#[derive(FromRow, Debug, Clone)]
pub struct Equipment {
    pub id: i32,
    pub item_id: i32,
    pub character_id: i32,
    pub position: i32,
    pub amount: i32,
    pub upgrade_slots: i32,
    pub level: i32,
    pub item_level: i32,
    pub exp: i32,
    pub str: i32,
    pub dex: i32,
    pub int: i32,
    pub luk: i32,
    pub hp: i32,
    pub mp: i32,
    pub w_atk: i32,
    pub m_atk: i32,
    pub w_def: i32,
    pub m_def: i32,
    pub acc: i32,
    pub avoid: i32,
    pub hands: i32,
    pub speed: i32,
    pub jump: i32,
    pub locked: i32,
    pub vicious: i32,
    pub owner: String,
    pub flag: i32,
}

impl Equipment {
    /// Loads a character's equipment
    pub async fn load_all(character_id: i32, db: &Db) -> anyhow::Result<Vec<Self>> {
        let equipment = sqlx::query_as::<_, Self>("SELECT * FROM equipment WHERE character_id = ?")
            .bind(character_id)
            .fetch_all(db)
            .await?;

        Ok(equipment)
    }
}
