use clap::{Parser, Subcommand};
use dotenvy::dotenv;
use rand::RngCore;
use slime_data::Config;
use sqlx::{
    mysql::{MySqlConnectOptions, MySqlPoolOptions},
    ConnectOptions, MySql, Pool, QueryBuilder,
};
use std::{env, str::FromStr};

type Db = Pool<MySql>;

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

    /// Initializes channels
    InitChannels,
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

    let config = Config::load();

    let cli = Cli::parse();

    match &cli.command {
        Commands::CreateAccount { name, password } => create_account(name, password, &pool).await?,
        Commands::InitChannels => init_channels(&config, &pool).await?,
    }

    Ok(())
}

/// Creates an account with the given name and password
async fn create_account(name: &str, password: &str, pool: &Db) -> anyhow::Result<()> {
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

/// Initializes the channels for each world defined in config.toml
async fn init_channels(config: &Config, db: &Db) -> anyhow::Result<()> {
    sqlx::query("DELETE FROM channels").execute(db).await?;

    let mut channels = Vec::new();

    for world in config.worlds.iter() {
        for i in 1..=world.channels {
            channels.push((i, world.name.clone(), world.id));
        }
    }

    let mut query_builder =
        QueryBuilder::<MySql>::new("INSERT INTO channels (id, world_name, world_id) ");

    query_builder.push_values(channels, |mut builder, channel| {
        builder.push_bind(channel.0);
        builder.push_bind(channel.1);
        builder.push_bind(channel.2);
    });
    query_builder.build().execute(db).await?;
    Ok(())
}
