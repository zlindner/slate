use crate::client::WorldClient;
use anyhow::Result;
use dashmap::DashMap;
use dotenv::dotenv;
use log::LevelFilter;
use oxy_core::{
    net::BroadcastPacket,
    prisma::{self, character, PrismaClient},
};
use simple_logger::SimpleLogger;
use std::sync::Arc;
use tokio::{net::TcpListener, sync::broadcast};

mod client;
mod handler;

pub struct Shared {
    maps: DashMap<i32, Map>,
}

impl Shared {
    pub fn new() -> Self {
        Self {
            maps: DashMap::new(),
        }
    }
}

pub struct Map {
    // TODO create MapleCharacter struct that contains character::Data and pos stuff
    // insert that into map instead
    // should ideally be a reference in the map that is owned in the client
    // can hopefully do with lifetimes
    pub characters: DashMap<i32, character::Data>,
}

impl Map {
    pub fn new() -> Self {
        Self {
            characters: DashMap::new(),
        }
    }
}

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
    let (tx, _rx) = broadcast::channel::<BroadcastPacket>(16);

    //oxy_core::nx::load_map(10000);

    let shared = Arc::new(Shared::new());

    loop {
        let (stream, _) = listener.accept().await?;
        session_id += 1;

        let client = WorldClient::new(
            stream,
            db.clone(),
            session_id,
            tx.clone(),
            tx.subscribe(),
            shared.clone(),
        );

        tokio::spawn(async move {
            client.process().await;
        });
    }
}

async fn startup(_db: &Arc<PrismaClient>) -> Result<()> {
    // TODO
    Ok(())
}
