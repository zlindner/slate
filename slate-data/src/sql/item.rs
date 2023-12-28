use sqlx::{Decode, Encode, FromRow};

use crate::Db;

#[derive(FromRow, Debug, Clone)]
pub struct Item {
    pub id: i32,
    pub item_id: i32,
    pub character_id: i32,
    pub inventory_type: InventoryType,
    pub position: i32,
    pub amount: i32,
    pub owner: String,
    pub flag: i32,
}

impl Item {
    /// Loads all of a character's items
    pub async fn load_all(character_id: i32, db: &Db) -> anyhow::Result<Vec<Self>> {
        let items = sqlx::query_as::<_, Item>("SELECT * FROM items WHERE character_id = ?")
            .bind(character_id)
            .fetch_all(db)
            .await?;

        Ok(items)
    }
}

#[derive(Decode, Encode, Clone, Copy, Debug)]
pub enum InventoryType {
    Equip,
    Use,
    Setup,
    Etc,
    Cash,
}

impl sqlx::Type<sqlx::MySql> for InventoryType {
    fn type_info() -> <sqlx::MySql as sqlx::Database>::TypeInfo {
        <str as sqlx::Type<sqlx::MySql>>::type_info()
    }

    fn compatible(ty: &<sqlx::MySql as sqlx::Database>::TypeInfo) -> bool {
        <str as sqlx::Type<sqlx::MySql>>::compatible(ty)
    }
}
