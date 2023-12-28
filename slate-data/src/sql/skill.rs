use sqlx::FromRow;

#[derive(FromRow, Debug, Clone)]
pub struct Skill {
    pub id: i32,
    pub character_id: i32,
    pub level: i32,
    pub mastery: i32,
    pub expiration: i64,
}

impl Skill {}

#[derive(FromRow, Debug, Clone)]
pub struct Cooldown {
    pub character_id: i32,
    pub skill_id: i32,
    pub start: i64,
    pub length: i64,
}

impl Cooldown {}
