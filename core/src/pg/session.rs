use sqlx::FromRow;

#[derive(FromRow)]
pub struct Session {
    pub id: i32,
    pub account_id: i32,
    pub character_id: i32,
    pub world_id: i32,
    pub channel_id: i16,
    pub login_attempts: i16,
    pub pin: String,
    pub pin_attempts: i16,
    pub pic: String,
    pub pic_attempts: i16,
}
