use dotenvy::dotenv;
use server::ChannelServer;
use sqlx::{
    mysql::{MySqlConnectOptions, MySqlPoolOptions},
    ConnectOptions,
};
use std::{env, str::FromStr};

mod packet_handler;
mod server;
mod session;
mod shutdown;
mod state;

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

    ChannelServer::start(pool).await?;
    Ok(())
}
