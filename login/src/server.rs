use crate::{client::Client, event_handler::EventHandler, Result};
use oxide_core::{net::codec::MapleCodec, pg::PgSession, util::Shutdown, Db, Error};
use std::sync::{
    atomic::{AtomicI32, Ordering},
    Arc,
};
use tokio::{
    io::AsyncWriteExt,
    net::TcpListener,
    signal,
    sync::{broadcast, mpsc},
};
use tokio_util::codec::Decoder;

pub struct ServerConfig {
    pub addr: String,
}

pub struct LoginServer {
    config: ServerConfig,
    events: Arc<EventHandler>,
}

impl LoginServer {
    pub async fn start(config: ServerConfig, db: Db) -> Result<()> {
        let server = LoginServer {
            config,
            events: Arc::new(EventHandler::new(db)),
        };

        server.events.on_start(&server.config.addr).await;

        let notify_shutdown: broadcast::Sender<()> = broadcast::channel(1).0;
        let (shutdown_complete_tx, mut shutdown_complete_rx): (
            mpsc::Sender<()>,
            mpsc::Receiver<()>,
        ) = mpsc::channel(1);

        tokio::select! {
            res = server.listen(&notify_shutdown) => {
                if let Err(e) = res {
                    log::error!("World server listen error: {}", e);
                }
            }
            _ = signal::ctrl_c() => {}
        }

        // send the shutdown signal to all subscribed tasks
        drop(notify_shutdown);

        // drop the final sender to the below receiver can complete
        drop(shutdown_complete_tx);

        // wait for all active connections to finish processing
        let _ = shutdown_complete_rx.recv().await;

        server.events.on_shutdown().await;
        Ok(())
    }

    async fn listen(&self, notify_shutdown: &broadcast::Sender<()>) -> Result<()> {
        let listener = TcpListener::bind(&self.config.addr).await?;
        let session_id = AtomicI32::new(0);

        loop {
            let (mut stream, _) = listener.accept().await?;
            let mut shutdown = Shutdown::new(notify_shutdown.subscribe());
            let events = self.events.clone();
            let session_id = session_id.fetch_add(1, Ordering::SeqCst);

            tokio::spawn(async move {
                let codec = MapleCodec::new();
                stream.write_all(&codec.handshake().bytes).await?;

                let mut client = Client {
                    stream: codec.framed(stream),
                    session: PgSession::new(session_id),
                    num_characters: 0,
                };

                events.on_connect(&mut client).await;

                while !shutdown.is_shutdown() {
                    let maybe_packet = tokio::select! {
                        res = client.read() => res?,
                        _ = shutdown.recv() => {
                            break;
                        }
                    };

                    // None => client disconnected
                    let packet = match maybe_packet {
                        Some(packet) => packet,
                        None => break,
                    };

                    events.on_packet(&mut client, packet).await;
                }

                events.on_disconnect(&mut client).await;
                return Ok::<(), Error>(());
            });
        }
    }
}
