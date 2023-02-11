use crate::{client::WorldClient, shared::Shared};
use anyhow::Result;
use dotenv::dotenv;
use log::LevelFilter;
use oxy_core::prisma::{self, PrismaClient};
use simple_logger::SimpleLogger;
use std::sync::Arc;
use tokio::net::TcpListener;

mod client;
mod handler;
mod map;
mod shared;

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

    // Parse addr from environment variables (defaults to 0.0.0.0:10000)
    let ip = std::env::var("WORLD_IP").unwrap_or("0.0.0.0".to_string());
    let port = std::env::var("WORLD_PORT").unwrap_or("10000".to_string());
    let addr = format!("{}:{}", ip, port);
    let listener = TcpListener::bind(&addr).await?;

    log::info!("World server started @ {}", addr);
    let mut session_id = 0;
    let shared = Arc::new(Shared::new(db));

    loop {
        let (stream, _) = listener.accept().await?;
        session_id += 1;

        let client = WorldClient::new(stream, session_id);
        let shared = shared.clone();

        tokio::spawn(async move {
            client.process(shared).await;
        });
    }
}

async fn startup(_db: &Arc<PrismaClient>) -> Result<()> {
    // TODO
    //let quest = oxy_core::nx::quest::load_quest(28337);
    //log::debug!("{:?}", quest);
    //oxy_core::nx::quest::load_quest(2300);

    Ok(())
}
