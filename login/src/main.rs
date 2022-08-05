use event_handler::LoginServerEventHandler;
use log::LevelFilter;
use oxide_core::{db, net::Server, Result};
use simple_logger::SimpleLogger;
use state::State;
use std::{env, sync::Arc};

mod event_handler;
mod packet_handler;
mod packets;
mod queries;
mod state;

#[tokio::main]
async fn main() -> Result<()> {
    dotenv::dotenv().ok();

    SimpleLogger::new()
        .with_module_level("tokio_util", LevelFilter::Debug)
        .with_module_level("mio", LevelFilter::Debug)
        .env()
        .init()
        .unwrap();

    let state = Arc::new(State::new());
    let db = db::new(10).await?;

    Server::new(
        env::var("LOGIN_SERVER_ADDR").unwrap(),
        LoginServerEventHandler::new(db, state),
    )
    .start()
    .await?;

    Ok(())
}
