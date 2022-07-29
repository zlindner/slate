use super::{codec::MapleCodec, Events, Packet};
use crate::{util::Shutdown, Error, Result};
use futures::TryStreamExt;
use std::sync::Arc;
use tokio::{
    net::{TcpListener, TcpStream},
    signal,
    sync::{broadcast, mpsc},
};
use tokio_util::codec::{Decoder, Framed};

pub struct Server {
    addr: String,
    events: Arc<Box<dyn Events>>,
}

impl Server {
    pub fn new<E: Events + 'static>(addr: String, events: E) -> Self {
        Self {
            addr,
            events: Arc::new(Box::new(events)),
        }
    }

    pub async fn start(&self) -> Result<()> {
        self.events.on_start(&self.addr).await;

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
            let mut shutdown = Shutdown::new(notify_shutdown.subscribe());
            let events = self.events.clone();

            tokio::spawn(async move {
                let mut stream = MapleCodec::new().framed(stream);
                events.on_connect(&mut stream).await;

                while !shutdown.is_shutdown() {
                    let maybe_packet = tokio::select! {
                        res = Self::read_packet(&mut stream) => res?,
                        _ = shutdown.recv() => {
                            return Ok::<(), Error>(());
                        }
                    };

                    // None => client disconnected
                    let packet = match maybe_packet {
                        Some(packet) => packet,
                        None => return Ok(()),
                    };

                    events.on_packet(&mut stream, packet).await;
                }

                Ok(())
            });
        }
    }

    async fn read_packet(stream: &mut Framed<TcpStream, MapleCodec>) -> Result<Option<Packet>> {
        loop {
            return Ok(stream.try_next().await?);
        }
    }
}
