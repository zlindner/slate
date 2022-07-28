use super::{Connection, HandlePacket};
use crate::{util::Shutdown, Result};
use tokio::{
    net::TcpListener,
    signal,
    sync::{broadcast, mpsc},
};

pub struct Server {
    addr: String,
}

impl Server {
    pub fn new(addr: String) -> Self {
        Self { addr }
    }

    pub async fn start(
        self,
        handler: impl HandlePacket + Send + Sync + Copy + 'static,
    ) -> Result<()> {
        log::info!("Server started @ {}", self.addr);

        let notify_shutdown: broadcast::Sender<()> = broadcast::channel(1).0;
        let (shutdown_complete_tx, mut shutdown_complete_rx): (
            mpsc::Sender<()>,
            mpsc::Receiver<()>,
        ) = mpsc::channel(1);

        tokio::select! {
            res = self.listen(&notify_shutdown, handler) => {
                if let Err(e) = res {
                    log::error!("Server listen error: {}", e);
                }
            }
            _ = signal::ctrl_c() => {
                log::info!("Server shutting down");
            }
        }

        // send the shutdown signal to all subscribed tasks
        drop(notify_shutdown);

        // drop the final sender to the below receiver can complete
        drop(shutdown_complete_tx);

        // wait for all active connections to finish processing
        let _ = shutdown_complete_rx.recv().await;

        Ok(())
    }

    async fn listen(
        self,
        notify_shutdown: &broadcast::Sender<()>,
        handler: impl HandlePacket + Send + Sync + Copy + 'static,
    ) -> Result<()> {
        let listener = TcpListener::bind(&self.addr).await?;

        loop {
            let (stream, _) = listener.accept().await?;
            // TODO call init(stream) or something to allow for handshake to be sent, pass fn as arg

            // create a shutdown listener for the current connection
            let shutdown = Shutdown::new(notify_shutdown.subscribe());

            tokio::spawn(async move {
                log::info!("Client connected to server");
                let mut connection = Connection::new(stream, shutdown);

                if let Err(e) = connection.read_packets(&handler).await {
                    log::error!("Connection error: {}", e);
                }

                log::info!("Client disconnected from server");
                if let Err(e) = connection.disconnect().await {
                    log::error!("Connection error: {}", e);
                }
            });
        }
    }
}
