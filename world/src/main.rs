use event_handler::WorldServerEventHandler;
use log::LevelFilter;
use oxide_core::{db, net::Server, redis, Result};
use simple_logger::SimpleLogger;
use std::env;

mod event_handler;
mod packet_handler;
mod packets;

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
    let redis = redis::new()?;

    Server::new(
        env::var("WORLD_SERVER_ADDR").unwrap(),
        WorldServerEventHandler::new(db, redis),
    )
    .start()
    .await?;

    Ok(())
}
