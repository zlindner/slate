use crate::{session::ChannelSession, shutdown::Shutdown};
use slime_data::sql;
use slime_net::MapleStream;
use sqlx::{MySql, Pool};
use std::{env, time::Instant};
use tokio::{
    net::TcpListener,
    sync::{broadcast, mpsc},
};

pub struct ChannelServer {
    pub data: sql::Channel,
    pub addr: String,
    pub db: Pool<MySql>,

    /// Broadcasts a shutdown signal to all active connections
    notify_shutdown: broadcast::Sender<()>,

    /// Used as part of the graceful shutdown process to wait for client
    /// connections to complete processing
    shutdown_complete_tx: mpsc::Sender<()>,
}

impl ChannelServer {
    /// Starts the channel server
    pub async fn start(db: Pool<MySql>) -> anyhow::Result<()> {
        let (channel, addr) = Self::get_available_channel(&db).await?;
        let (notify_shutdown, _) = broadcast::channel::<()>(1);
        let (shutdown_complete_tx, mut shutdown_complete_rx) = mpsc::channel::<()>(1);

        let mut server = ChannelServer {
            data: channel,
            addr,
            db,
            notify_shutdown,
            shutdown_complete_tx,
        };

        server.execute_startup_tasks().await?;

        tokio::select! {
            _ = server.listen() => {},
            _ = tokio::signal::ctrl_c() => {}
        }

        server.execute_shutdown_tasks().await?;

        let ChannelServer {
            shutdown_complete_tx,
            notify_shutdown,
            ..
        } = server;

        // When `notify_shutdown` is dropped, all tasks which have `subscribe`d will
        // receive the shutdown signal and can exit
        drop(notify_shutdown);

        // Drop final `Sender` so the `Receiver` below can complete
        drop(shutdown_complete_tx);

        // Wait for all active connections to finish processing
        let _ = shutdown_complete_rx.recv().await;
        Ok(())
    }

    async fn listen(&mut self) -> anyhow::Result<()> {
        let listener = TcpListener::bind(&self.addr).await?;
        log::info!(
            "{} {} started @ {}",
            self.data.world_name,
            self.data.id,
            &self.addr
        );

        let mut session_id = 0;

        loop {
            let stream = match listener.accept().await {
                Ok((stream, _)) => MapleStream::new(stream),
                Err(e) => {
                    log::error!("Error accepting connection: {}", e);
                    continue;
                }
            };

            session_id += 1;

            let session = ChannelSession {
                id: session_id,
                stream,
                db: self.db.clone(),
                shutdown: Shutdown::new(self.notify_shutdown.subscribe()),
                _shutdown_complete: self.shutdown_complete_tx.clone(),
            };

            // Spawn a task for handling the new login session
            tokio::spawn(async move {
                session.handle().await;
            });
        }
    }

    /// Executes startup tasks
    async fn execute_startup_tasks(&self) -> anyhow::Result<()> {
        let start = Instant::now();
        log::info!("Executing startup tasks...");

        sqlx::query("UPDATE channels SET is_online = 1 WHERE id = ? AND world_id = ?")
            .bind(self.data.id)
            .bind(self.data.world_id)
            .execute(&self.db)
            .await?;

        log::info!("Finished startup tasks in {:?}", start.elapsed());
        Ok(())
    }

    /// Executes shutdown tasks
    async fn execute_shutdown_tasks(&self) -> anyhow::Result<()> {
        let start = Instant::now();
        log::info!("Executing shutdown tasks...");

        sqlx::query("UPDATE channels SET is_online = 0 WHERE id = ? AND world_id = ?")
            .bind(self.data.id)
            .bind(self.data.world_id)
            .execute(&self.db)
            .await?;

        log::info!("Finished shutdown tasks in {:?}", start.elapsed());
        Ok(())
    }

    /// Gets the first available address to listen on
    async fn get_available_channel(db: &Pool<MySql>) -> anyhow::Result<(sql::Channel, String)> {
        let channel =
            sqlx::query_as::<_, sql::Channel>("SELECT * FROM channels WHERE is_online = false")
                .fetch_one(db)
                .await?;

        // TODO handle no channel being available

        let ip = env::var("CHANNEL_IP")?;
        let base_port: i32 = env::var("CHANNEL_BASE_PORT")?.parse()?;

        // ex. 10000 + ((1 - 1) * 1000) + (1 - 1) = 10000 (world 1, channel 1)
        // ex. 10000 + ((2 - 1) * 1000) + (2 - 1) = 11001 (world 2, channel 2)
        let port = base_port + ((channel.world_id - 1) * 1000) + (channel.id - 1);
        let addr = format!("{}:{}", ip, port);
        Ok((channel, addr))
    }
}
