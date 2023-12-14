use config::Config;
use dotenvy::dotenv;
use server::LoginServer;
use slime_data::sql::account::LoginState;
use sqlx::{
    mysql::{MySqlConnectOptions, MySqlPoolOptions},
    ConnectOptions, MySql, Pool,
};
use std::{env, str::FromStr, sync::Arc, time::Instant};

mod config;
mod packet;
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

    // TODO move to server
    startup(&pool).await?;

    let server = LoginServer {
        addr: env::var("LOGIN_ADDR")?,
        db: pool,
        config: Arc::new(Config::load()),
    };
    server.start().await?;

    Ok(())
}

/// Execute startup tasks
async fn startup(pool: &Pool<MySql>) -> anyhow::Result<()> {
    let start = Instant::now();
    log::info!("Executing startup tasks...");

    sqlx::query("UPDATE accounts SET state = ?")
        .bind(LoginState::LoggedOut)
        .execute(pool)
        .await?;

    sqlx::query("UPDATE worlds SET connected_players = ?")
        .bind(0)
        .execute(pool)
        .await?;

    sqlx::query("DELETE FROM login_sessions")
        .execute(pool)
        .await?;

    log::info!("Finished startup tasks in {:?}", start.elapsed());
    Ok(())
}
