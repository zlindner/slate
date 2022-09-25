use sqlx::FromRow;

#[derive(FromRow, Default)]
pub struct PgSession {
    pub id: i32,
    pub account_id: i32,
    pub character_id: i32,
    pub world_id: i16,
    pub channel_id: i16,
    #[sqlx(default)]
    pub login_attempts: i16,
    #[sqlx(default)]
    pub pin: String,
    #[sqlx(default)]
    pub pin_attempts: i16,
    #[sqlx(default)]
    pub pic: String,
    #[sqlx(default)]
    pub pic_attempts: i16,
}

impl PgSession {
    pub fn new(session_id: i32) -> Self {
        let mut session = Self::default();
        session.id = session_id;
        session
    }
}
