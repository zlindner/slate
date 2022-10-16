use anyhow::Result;
use dotenv::dotenv;
use handler::EventHandler;
use log::LevelFilter;
use oxy_core::{
    net::Server,
    prisma::{self, PrismaClient},
};
use simple_logger::SimpleLogger;
use std::sync::Arc;

mod handler;

#[tokio::main]
async fn main() -> Result<()> {
    dotenv().ok();

    SimpleLogger::new()
        .with_module_level("mio", LevelFilter::Off)
        .env()
        .init()?;

    let db: Arc<PrismaClient> = Arc::new(prisma::new_client().await?);
    Server::start("0.0.0.0:8484", EventHandler, db).await?;

    Ok(())
}
