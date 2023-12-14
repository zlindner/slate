use crate::Db;
use sqlx::FromRow;

#[derive(FromRow)]
pub struct World {
    pub id: i32,
    pub connected_players: i32,
}

impl World {
    /// Loads a sql::World from the database
    pub async fn load(id: i32, db: &Db) -> anyhow::Result<World> {
        let world = sqlx::query_as::<_, Self>("SELECT * FROM worlds WHERE id = ?")
            .bind(id)
            .fetch_one(db)
            .await?;

        Ok(world)
    }
}
