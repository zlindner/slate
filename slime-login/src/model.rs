use sqlx::{
    types::chrono::{DateTime, Utc},
    Decode, Encode, FromRow,
};

#[derive(Default, FromRow)]
pub struct Session {
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
    pub accepted_tos: bool,
}

#[derive(FromRow)]
pub struct Account {
    pub id: i32,
    pub name: String,
    pub password: String,
    pub pin: String,
    pub pic: String,
    pub state: LoginState,
    pub banned: bool,
    pub accepted_tos: bool,
    pub last_login: DateTime<Utc>,
    pub gender: i32,
}

#[derive(Debug, Decode, Encode)]
pub enum LoginState {
    LoggedIn,
    Transitioning,
    LoggedOut,
}

impl sqlx::Type<sqlx::MySql> for LoginState {
    fn type_info() -> <sqlx::MySql as sqlx::Database>::TypeInfo {
        <str as sqlx::Type<sqlx::MySql>>::type_info()
    }

    fn compatible(ty: &<sqlx::MySql as sqlx::Database>::TypeInfo) -> bool {
        <str as sqlx::Type<sqlx::MySql>>::compatible(ty)
    }
}
