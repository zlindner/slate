use sqlx::{Decode, Encode, FromRow};

#[derive(FromRow, Debug, Clone)]
pub struct Quest {
    pub id: i32,
    pub character_id: i32,
    pub status: QuestStatus,
    pub time: i32,
    pub expires: i32,
    pub forfeited: i32,
    pub completed: i32,
    pub info: i32,
}

#[derive(Decode, Encode, Debug, Copy, Clone)]
pub enum QuestStatus {
    NotStarted,
    Started,
    Completed,
}

impl sqlx::Type<sqlx::MySql> for QuestStatus {
    fn type_info() -> <sqlx::MySql as sqlx::Database>::TypeInfo {
        <str as sqlx::Type<sqlx::MySql>>::type_info()
    }

    fn compatible(ty: &<sqlx::MySql as sqlx::Database>::TypeInfo) -> bool {
        <str as sqlx::Type<sqlx::MySql>>::compatible(ty)
    }
}
