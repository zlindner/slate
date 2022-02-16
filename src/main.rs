mod client;
mod crypto;
mod login;
mod net;
mod world;

use deadpool_postgres::{Manager, Pool};
use dotenv::dotenv;
use log::LevelFilter;
use login::server::LoginServer;
use openssl::ssl::{SslConnector, SslMethod, SslVerifyMode};
use postgres_openssl::MakeTlsConnector;
use simple_logger::SimpleLogger;
use std::env;
use std::error::Error;
use std::sync::Arc;
use tokio::sync::Mutex;
use world::World;

pub struct Server {
    worlds: Vec<World>,
}

impl Server {
    fn new() -> Self {
        Server { worlds: Vec::new() }
    }

    fn load_worlds(&mut self) {
        let toml = std::fs::read_to_string("config/worlds.toml").unwrap();
        let config: world::Config = toml::from_str(&toml).unwrap();

        for world_config in config.worlds.into_iter() {
            self.worlds.push(World::from_config(world_config));
        }
    }
}

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
    pg_config.host(&env::var("DATABASE_HOST").unwrap());
    pg_config.dbname(&env::var("DATABASE_NAME").unwrap());
    pg_config.user(&env::var("DATABASE_USER").unwrap());
    pg_config.password(&env::var("DATABASE_PASSWORD").unwrap());

    let mut ssl_builder = SslConnector::builder(SslMethod::tls()).unwrap();
    ssl_builder.set_verify(SslVerifyMode::NONE);
    let connector = MakeTlsConnector::new(ssl_builder.build());

    let manager = Manager::new(pg_config, connector);
    let pool = Pool::builder(manager).max_size(10).build().unwrap();

    let server = Arc::new(Mutex::new(Server::new()));
    server.lock().await.load_worlds();

    LoginServer::new().start(&server, &pool).await?;

    Ok(())
}
