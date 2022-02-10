use crate::client::{Client, ClientType};

use deadpool_postgres::Pool;
use std::error::Error;
use tokio::net::TcpListener;

pub struct LoginServer;

impl LoginServer {
    pub fn new() -> Self {
        LoginServer {}
    }

    pub async fn start(self, pool: &Pool) -> Result<(), Box<dyn Error>> {
        let listener = TcpListener::bind("127.0.0.1:8484").await?;
        log::info!("Login server started on port 8484");

        loop {
            let (stream, addr) = listener.accept().await?;
            let pool = pool.clone();

            tokio::spawn(async move {
                let client = Client::new(stream, addr, pool, ClientType::Login);

                if let Err(e) = client.connect().await {
                    log::error!("An error occurred while connecting with client: {}", e);
                }
            });
        }
    }
}
