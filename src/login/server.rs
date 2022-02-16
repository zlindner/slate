use crate::{
    client::{Client, ClientType},
    Server,
};

use deadpool_postgres::Pool;
use std::{error::Error, sync::Arc};
use tokio::{net::TcpListener, sync::Mutex};

pub struct LoginServer;

impl LoginServer {
    pub fn new() -> Self {
        LoginServer {}
    }

    pub async fn start(
        self,
        server: &Arc<Mutex<Server>>,
        pool: &Pool,
    ) -> Result<(), Box<dyn Error>> {
        let listener = TcpListener::bind("127.0.0.1:8484").await?;
        log::info!("Login server started on port 8484");

        loop {
            let (stream, addr) = listener.accept().await?;
            let server = Arc::clone(server);
            let pool = pool.clone();

            tokio::spawn(async move {
                let client = Client::new(stream, addr, server, pool, ClientType::Login);

                if let Err(e) = client.connect().await {
                    log::error!("An error occurred while connecting with client: {}", e);
                }
            });
        }
    }
}
