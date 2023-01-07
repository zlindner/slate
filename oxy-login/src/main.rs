use crate::client::LoginClient;
use anyhow::Result;
use dotenv::dotenv;
use log::LevelFilter;
use oxy_core::prisma::{self, account, world, LoginState, PrismaClient};
use simple_logger::SimpleLogger;
use std::sync::Arc;
use tokio::net::TcpListener;

mod client;
mod handler;

#[tokio::main]
async fn main() -> Result<()> {
    dotenv().ok();

    SimpleLogger::new()
        .with_level(LevelFilter::Debug)
        .with_module_level("quaint", LevelFilter::Off)
        .with_module_level("mobc", LevelFilter::Off)
        .with_module_level("tokio_postgres", LevelFilter::Error)
        .env()
        .init()?;

    // Initialize db and perform startup operations
    let db: Arc<PrismaClient> = Arc::new(prisma::new_client().await?);
    startup(&db).await?;

    // Parse addr from environment variables (defaults to 0.0.0.0:8484)
    let ip = std::env::var("LOGIN_IP").unwrap_or("0.0.0.0".to_string());
    let port = std::env::var("LOGIN_PORT").unwrap_or("8484".to_string());
    let addr = format!("{}:{}", ip, port);
    let listener = TcpListener::bind(&addr).await?;

    log::info!("Login server started @ {}", addr);
    let mut session_id = 0;

    loop {
        let (stream, _) = listener.accept().await?;
        session_id += 1;
        let client = LoginClient::new(stream, db.clone(), session_id);

        tokio::spawn(async move {
            client.process().await;
        });
    }
}

async fn startup(db: &Arc<PrismaClient>) -> Result<()> {
    // Clear all sessions
    db.session().delete_many(vec![]).exec().await?;

    // Set all accounts to logged out
    db.account()
        .update_many(vec![], vec![account::state::set(LoginState::LoggedOut)])
        .exec()
        .await?;

    // Set connected players count to 0 for each world
    db.world()
        .update_many(vec![], vec![world::connected::set(0)])
        .exec()
        .await?;

    Ok(())
}
