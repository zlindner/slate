use dotenvy::dotenv;
use server::LoginServer;
use slime_data::Config;
use sqlx::{
    mysql::{MySqlConnectOptions, MySqlPoolOptions},
    ConnectOptions,
};
use std::{env, str::FromStr, sync::Arc};

mod packet_handler;
mod server;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv().ok();
    env_logger::init();

    let db_url = env::var("DATABASE_URL")?;
    let options = MySqlConnectOptions::from_str(&db_url)?.disable_statement_logging();
    let pool = MySqlPoolOptions::new()
        .min_connections(5)
        .max_connections(100)
        .connect_with(options)
        .await?;

    let server = LoginServer {
        addr: env::var("LOGIN_ADDR")?,
        db: pool,
        config: Arc::new(Config::load()),
    };
    server.start().await?;

    Ok(())
}
