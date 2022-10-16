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

    let port = std::env::var("PORT").unwrap_or("8484".to_string());
    let addr = format!("0.0.0.0:{}", port);
    Server::start(&addr, EventHandler, db).await?;

    Ok(())
}
