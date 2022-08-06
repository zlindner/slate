use sqlx::{
    postgres::{PgHasArrayType, PgTypeInfo},
    FromRow, Type,
};

#[derive(FromRow, Type)]
pub struct Pet {
    pub id: i32,
    pub item_id: i32,
}

impl PgHasArrayType for Pet {
    fn array_type_info() -> sqlx::postgres::PgTypeInfo {
        PgTypeInfo::with_name("simple")
    }
}
