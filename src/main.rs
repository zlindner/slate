mod client;
mod login;
mod maple_aes;
mod maple_codec;
mod packet;
mod shanda;

use deadpool_postgres::{Manager, Pool};
use dotenv::dotenv;
use log::LevelFilter;
use login::server::LoginServer;
use simple_logger::SimpleLogger;
use std::env;
use std::error::Error;
use tokio_postgres::NoTls;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // load environment variables from .env
    dotenv().ok();

    SimpleLogger::new()
        .with_module_level("tokio_util", LevelFilter::Debug)
        .with_module_level("mio", LevelFilter::Debug)
        .with_module_level("tokio_postgres", LevelFilter::Debug)
        .env()
        .init()
        .unwrap();

    let mut pg_config = tokio_postgres::Config::new();
    pg_config.user(&env::var("DATABASE_USER").unwrap());
    pg_config.password(&env::var("DATABASE_PASSWORD").unwrap());
    pg_config.dbname(&env::var("DATABASE_NAME").unwrap());
    pg_config.host("localhost");

    let manager = Manager::new(pg_config, NoTls);
    let pool = Pool::builder(manager).max_size(10).build().unwrap();

    LoginServer::new().start(&pool).await?;

    Ok(())
}
