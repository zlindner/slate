use crate::Db;
use sqlx::FromRow;

#[derive(FromRow)]
pub struct LoginSession {
    // TODO this is always -1 on login server side, kinda weird?
    pub id: i32,
    pub account_id: i32,
    pub character_id: i32,
    pub world_id: i32,
    pub channel_id: i32,
    #[sqlx(default)]
    pub login_attempts: i32,
    #[sqlx(default)]
    pub pin: String,
    #[sqlx(default)]
    pub pin_attempts: i32,
    #[sqlx(default)]
    pub pic: String,
    #[sqlx(default)]
    pub pic_attempts: i32,
}

impl LoginSession {
    /// Loads a login session with the given id if it exists
    pub async fn load_optional(id: i32, db: &Db) -> anyhow::Result<Option<Self>> {
        let session =
            sqlx::query_as::<_, LoginSession>("SELECT * FROM login_sessions WHERE id = ?")
                .bind(id)
                .fetch_optional(db)
                .await?;

        Ok(session)
    }

    /// Deletes a login session with the given id
    pub async fn delete(&self, db: &Db) -> anyhow::Result<()> {
        sqlx::query("DELETE FROM login_sessions WHERE id = ?")
            .bind(self.id)
            .execute(db)
            .await?;

        Ok(())
    }
}

impl Default for LoginSession {
    fn default() -> Self {
        Self {
            id: -1,
            account_id: -1,
            character_id: -1,
            world_id: -1,
            channel_id: -1,
            login_attempts: 0,
            pin: String::new(),
            pin_attempts: 0,
            pic: String::new(),
            pic_attempts: 0,
        }
    }
}
