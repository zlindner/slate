use clap::{Parser, Subcommand};
use dotenvy::dotenv;
use rand::RngCore;
use sqlx::{
    mysql::{MySqlConnectOptions, MySqlPoolOptions},
    ConnectOptions, MySql, Pool,
};
use std::{env, str::FromStr};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Creates an account
    CreateAccount { name: String, password: String },
}

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    dotenv().ok();
    env_logger::init();

    let db_url = env::var("DATABASE_URL")?;
    let options = MySqlConnectOptions::from_str(&db_url)?.disable_statement_logging();
    let pool = MySqlPoolOptions::new()
        .min_connections(1)
        .max_connections(1)
        .connect_with(options)
        .await?;

    let cli = Cli::parse();

    match &cli.command {
        Commands::CreateAccount { name, password } => create_account(name, password, &pool).await?,
    }

    Ok(())
}

async fn create_account(name: &str, password: &str, pool: &Pool<MySql>) -> anyhow::Result<()> {
    log::info!("Creating account {}", name);

    // Generate random salt
    let mut salt = [0u8; 64];
    rand::thread_rng().fill_bytes(&mut salt);

    let config = argon2::Config::original();

    // Hash password
    let hash = argon2::hash_encoded(password.as_bytes(), &salt, &config).unwrap();

    log::info!("Generated hash of len: {}", hash.len());

    sqlx::query(
        "INSERT INTO accounts (name, password) 
        VALUES (?, ?)",
    )
    .bind(name)
    .bind(hash)
    .execute(pool)
    .await?;

    Ok(())
}
