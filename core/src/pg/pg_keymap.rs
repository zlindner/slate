use sqlx::FromRow;

#[derive(FromRow)]
pub struct PgKeymap {
    pub character_id: i32,
    pub key: i32,
    #[sqlx(rename = "type")]
    pub _type: i32,
    pub action: i32,
}
