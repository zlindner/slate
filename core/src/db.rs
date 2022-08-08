use super::Result;
use sqlx::{
    postgres::{PgConnectOptions, PgPoolOptions},
    Pool, Postgres,
};
use std::env;

pub type Db = Pool<Postgres>;

pub async fn new(max_connections: u32) -> Result<Db> {
    let options = PgConnectOptions::new()
        .host(&env::var("DATABASE_HOST").unwrap())
        .database(&env::var("DATABASE_NAME").unwrap())
        .username(&env::var("DATABASE_USER").unwrap())
        .password(&env::var("DATABASE_PASSWORD").unwrap());

    let pool = PgPoolOptions::new()
        .max_connections(max_connections)
        .connect_with(options)
        .await?;

    Ok(pool)
}
