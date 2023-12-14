use crate::Db;
use sqlx::FromRow;

#[derive(FromRow)]
pub struct Channel {
    pub id: i32,
    pub world_name: String,
    pub world_id: i32,
    pub is_online: bool,
    pub connected_players: i32,
}

impl Channel {
    /// Loads a channel from the database
    pub async fn load(id: i32, db: &Db) -> anyhow::Result<Self> {
        let channel = sqlx::query_as::<_, Self>("SELECT * FROM channels WHERE id = ?")
            .bind(id)
            .fetch_one(db)
            .await?;

        Ok(channel)
    }
}
