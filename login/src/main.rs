mod handler;

use handler::LoginServerHandler;
use log::LevelFilter;
use oxide_core::{net::Server, Result};
use simple_logger::SimpleLogger;
use std::env;

#[tokio::main]
async fn main() -> Result<()> {
    dotenv::dotenv().ok();

    SimpleLogger::new()
        .with_module_level("tokio_util", LevelFilter::Debug)
        .with_module_level("mio", LevelFilter::Debug)
        .env()
        .init()
        .unwrap();

    Server::new(env::var("LOGIN_SERVER_ADDR").unwrap())
        .start(LoginServerHandler)
        .await?;

    Ok(())
}
