use anyhow::Result;
use dotenv::dotenv;
use handler::PacketHandler;
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
        .with_level(LevelFilter::Debug)
        .with_module_level("quaint", LevelFilter::Off)
        .with_module_level("mobc", LevelFilter::Off)
        .with_module_level("tokio_postgres", LevelFilter::Error)
        .env()
        .init()?;

    let port = std::env::var("PORT").unwrap_or("8484".to_string());
    let addr = format!("0.0.0.0:{}", port);
    let db: Arc<PrismaClient> = Arc::new(prisma::new_client().await?);

    startup(&db).await?;
    Server::start(&addr, &PacketHandler, db).await?;

    Ok(())
}

async fn startup(db: &Arc<PrismaClient>) -> Result<()> {
    // TODO set all accounts to logged out, delete sessions, etc.
    Ok(())
}
