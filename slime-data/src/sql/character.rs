use crate::Db;
use sqlx::{
    types::chrono::{DateTime, Utc},
    FromRow, Row,
};

#[derive(FromRow)]
pub struct Character {
    pub id: i32,
    pub account_id: i32,
    pub world_id: i32,
    pub name: String,
    pub level: i32,
    pub exp: i32,
    pub gacha_exp: i32,
    pub str: i32,
    pub dex: i32,
    pub luk: i32,
    pub int: i32,
    pub hp: i32,
    pub mp: i32,
    pub max_hp: i32,
    pub max_mp: i32,
    pub mesos: i32,
    pub job: i32,
    pub skin_colour: i32,
    pub gender: i32,
    pub fame: i32,
    pub hair: i32,
    pub face: i32,
    pub ap: i32,
    pub sp: String,
    pub map: i32,
    pub spawn_point: i32,
    pub gm: i32,
    pub party: Option<i32>,
    pub buddy_capacity: i32,
    pub created_at: DateTime<Utc>,
    pub rank: i32,
    pub rank_move: i32,
    pub job_rank: i32,
    pub job_rank_move: i32,
    pub guild: Option<i32>,
    pub guild_rank: Option<i32>,
    pub equip_slots: i32,
    pub use_slots: i32,
    pub setup_slots: i32,
    pub etc_slots: i32,
    pub cash_slots: i32,
}

impl Character {
    /// Loads a character by name in the selected world
    pub async fn load_by_name(name: &String, world_id: i32, db: &Db) -> anyhow::Result<Self> {
        let character =
            sqlx::query_as::<_, Self>("SELECT * FROM characters WHERE name = ? AND world_id = ?")
                .bind(name)
                .bind(world_id)
                .fetch_one(db)
                .await?;

        Ok(character)
    }

    /// Loads alls characters by account id in the selected world
    pub async fn load_all(account_id: i32, world_id: i32, db: &Db) -> anyhow::Result<Vec<Self>> {
        let characters = sqlx::query_as::<_, Self>(
            "SELECT * FROM characters WHERE account_id = ? AND world_id = ?",
        )
        .bind(account_id)
        .bind(world_id)
        .fetch_all(db)
        .await?;

        Ok(characters)
    }

    /// Get the number of characters an account has in the selected world
    pub async fn get_count(account_id: i32, world_id: i32, db: &Db) -> anyhow::Result<i32> {
        let num_characters: i32 = sqlx::query(
            "SELECT COUNT(*) AS count FROM characters WHERE account_id = ? AND world_id = ?",
        )
        .bind(account_id)
        .bind(world_id)
        .fetch_one(db)
        .await?
        .get("count");

        Ok(num_characters)
    }
}
