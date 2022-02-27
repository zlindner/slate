use crate::{client::Client, db, net::connection::Connection, shutdown::Shutdown, Result, Shared};

use std::{
    future::Future,
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    },
};
use tokio::{
    net::TcpListener,
    sync::{broadcast, mpsc},
};

#[derive(Debug)]
struct Listener {
    listener: TcpListener,
    notify_shutdown: broadcast::Sender<()>,
    shutdown_complete_rx: mpsc::Receiver<()>,
    shutdown_complete_tx: mpsc::Sender<()>,
    shared: Arc<Shared>,
}

pub async fn start(listener: TcpListener, shutdown: impl Future, shared: &Arc<Shared>) {
    let (notify_shutdown, _) = broadcast::channel(1);
    let (shutdown_complete_tx, shutdown_complete_rx) = mpsc::channel(1);

    let mut server = Listener {
        listener,
        notify_shutdown,
        shutdown_complete_tx,
        shutdown_complete_rx,
        shared: Arc::clone(shared),
    };

    tokio::select! {
        res = server.start() => {
            if let Err(e) = res {
                log::error!("Login server failed to accept connection: {}", e);
            }
        }
        _ = shutdown => {
            log::info!("Login server shutting down");
        }
    }

    // extract the shutdown_complete receiver and sender, explicitly drop shutdown_transmitter
    let Listener {
        mut shutdown_complete_rx,
        shutdown_complete_tx,
        notify_shutdown,
        ..
    } = server;

    // send the shutdown signal to all subscribed tasks
    drop(notify_shutdown);

    // drop the final sender to the below receiver can complete
    drop(shutdown_complete_tx);

    // wait for all active connections to finish processing
    let _ = shutdown_complete_rx.recv().await;
}

impl Listener {
    async fn start(&mut self) -> Result<()> {
        log::info!("Login server started on port 8484");

        let session_id = AtomicUsize::new(0);
        let db = db::new().await?;

        loop {
            // accept() returns a (TcpStream, SockAddr), ignore the SockAddr for now
            let socket = self.listener.accept().await?.0;

            let mut client = Client {
                session_id: session_id.fetch_add(1, Ordering::SeqCst),
                db: db.clone(),
                connection: Connection::new(socket).await?,
                shutdown: Shutdown::new(self.notify_shutdown.subscribe()),
                shared: Arc::clone(&self.shared),
                login_attempts: 0,
                id: -1,
            };

            tokio::spawn(async move {
                if let Err(e) = client.connect().await {
                    log::error!("Client connection error: {}", e);
                }

                if let Err(e) = client.on_disconnect().await {
                    log::error!("Client on_disconnect() hook failed: {}", e);
                }

                log::info!("Client disconnected from login server");
            });
        }
    }
}
