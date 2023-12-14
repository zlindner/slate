use sqlx::FromRow;

#[derive(FromRow)]
pub struct LoginSession {
    pub account_id: i32,
    pub character_id: i32,
    pub world_id: i32,
    pub channel_id: i32,
    pub map_id: i32,
    pub login_attempts: i32,
    pub pin: String,
    pub pin_attempts: i32,
    pub pic: String,
    pub pic_attempts: i32,
}

impl LoginSession {}

impl Default for LoginSession {
    fn default() -> Self {
        Self {
            account_id: -1,
            character_id: -1,
            world_id: -1,
            channel_id: -1,
            map_id: -1,
            login_attempts: 0,
            pin: String::new(),
            pin_attempts: 0,
            pic: String::new(),
            pic_attempts: 0,
        }
    }
}
