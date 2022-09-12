use log::LevelFilter;
use oxide_core::{db, Result};
use server::{ServerConfig, WorldServer};
use simple_logger::SimpleLogger;
use std::env;

mod client;
mod event_handler;
mod packet_handler;
mod packets;
mod server;

#[tokio::main]
async fn main() -> Result<()> {
    dotenv::dotenv().ok();

    SimpleLogger::new()
        .with_module_level("tokio_util", LevelFilter::Debug)
        .with_module_level("mio", LevelFilter::Debug)
        .env()
        .init()
        .unwrap();

    let db = db::new(10).await?;

    let server_config = ServerConfig {
        addr: env::var("WORLD_SERVER_ADDR").unwrap(),
    };

    WorldServer::start(server_config, db).await?;

    Ok(())
}
