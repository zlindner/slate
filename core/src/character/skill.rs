use sqlx::{
    postgres::{PgHasArrayType, PgTypeInfo},
    FromRow, Type,
};

#[derive(FromRow, Type)]
pub struct Skill {
    pub skill_id: i32,
    pub level: i32,
    pub mastery_level: i32,
    pub expiration: i64,
}

impl Skill {
    pub fn is_fourth_job(&self) -> bool {
        let job = self.skill_id / 10000;

        match job {
            2212 => false,
            2217001 | 22171003 | 22171004 | 2218100 | 22181003 => true,
            _ => job % 10 == 2,
        }
    }
}

impl PgHasArrayType for Skill {
    fn array_type_info() -> sqlx::postgres::PgTypeInfo {
        PgTypeInfo::with_name("simple")
    }
}

#[derive(FromRow, Type)]
pub struct Cooldown {
    pub skill_id: i32,
    pub start_time: i64,
    pub length: i64,
}

impl PgHasArrayType for Cooldown {
    fn array_type_info() -> sqlx::postgres::PgTypeInfo {
        PgTypeInfo::with_name("simple")
    }
}
