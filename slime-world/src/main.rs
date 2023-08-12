use dotenvy::dotenv;
use server::WorldServer;
use sqlx::mysql::MySqlPoolOptions;
use std::env;

mod server;

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    dotenv().ok();

    let db_url = env::var("DATABASE_URL")?;
    let pool = MySqlPoolOptions::new()
        .min_connections(5)
        .max_connections(100)
        .connect(&db_url)
        .await?;

    let server = WorldServer::new();

    Ok(())
}
