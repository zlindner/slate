use super::{cipher::Cipher, codec::MapleCodec};
use crate::{util::Shutdown, Result};
use futures::SinkExt;
use tokio::{
    net::{TcpListener, TcpStream},
    signal,
    sync::{broadcast, mpsc},
};
use tokio_stream::StreamExt;
use tokio_util::codec::{Decoder, Framed};

pub struct TcpServer {
    addr: String,
}

impl TcpServer {
    pub fn new(addr: String) -> Self {
        Self { addr }
    }

    pub async fn start(&self) -> Result<()> {
        log::info!("Server started @ {}", self.addr);

        let notify_shutdown: broadcast::Sender<()> = broadcast::channel(1).0;
        let (shutdown_complete_tx, mut shutdown_complete_rx): (
            mpsc::Sender<()>,
            mpsc::Receiver<()>,
        ) = mpsc::channel(1);

        tokio::select! {
            res = self.listen(&notify_shutdown) => {
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

    async fn listen(&self, notify_shutdown: &broadcast::Sender<()>) -> Result<()> {
        let listener = TcpListener::bind(&self.addr).await?;

        loop {
            let (stream, _) = listener.accept().await?;
            // TODO call init(stream) or something to allow for handshake to be sent, pass fn as arg

            // create a shutdown listener for the current connection
            let shutdown = Shutdown::new(notify_shutdown.subscribe());

            tokio::spawn(async move {
                let send = Cipher::new(0xffff - 83);
                let recv = Cipher::new(83);
                let mut maple_stream = MapleCodec::new(send, recv).framed(stream);

                if let Err(e) = Self::connect(&mut maple_stream, shutdown).await {
                    log::error!("Client connection error: {}", e);
                }

                if let Err(e) = Self::disconnect(&mut maple_stream).await {
                    log::error!("Client disconnect error: {}", e);
                }
            });
        }
    }

    async fn connect(
        stream: &mut Framed<TcpStream, MapleCodec>,
        mut shutdown: Shutdown,
    ) -> Result<()> {
        log::info!("Client connected to server");

        while !shutdown.is_shutdown() {
            let maybe_packet = tokio::select! {
                res = stream.try_next() => res?,
                _ = shutdown.recv() => {
                    return Ok(());
                }
            };

            // None => client disconnected
            let packet = match maybe_packet {
                Some(packet) => packet,
                None => return Ok(()),
            };

            log::debug!("Received packet: {}", packet);
        }

        Ok(())
    }

    async fn disconnect(stream: &mut Framed<TcpStream, MapleCodec>) -> Result<()> {
        log::info!("Client disconnected from server");
        // TODO don't know if this is right
        stream.close().await?;
        Ok(())
    }
}
