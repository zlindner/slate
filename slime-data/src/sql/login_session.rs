use crate::Db;
use sqlx::FromRow;

#[derive(FromRow)]
pub struct LoginSession {
    pub character_id: i32,
    pub world_id: i32,
    pub channel_id: i32,
    pub map_id: i32,
    #[sqlx(default)]
    pub account_id: i32,
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
    pub async fn load_optional(session_id: i32, db: &Db) -> anyhow::Result<Option<Self>> {
        let session =
            sqlx::query_as::<_, LoginSession>("SELECT * FROM login_sessions WHERE id = ?")
                .bind(session_id)
                .fetch_optional(db)
                .await?;

        Ok(session)
    }
}

impl Default for LoginSession {
    fn default() -> Self {
        Self {
            character_id: -1,
            world_id: -1,
            channel_id: -1,
            map_id: -1,
            account_id: -1,
            login_attempts: 0,
            pin: String::new(),
            pin_attempts: 0,
            pic: String::new(),
            pic_attempts: 0,
        }
    }
}
