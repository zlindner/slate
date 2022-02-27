use tokio::sync::broadcast;

#[derive(Debug)]
pub struct Shutdown {
    // flag indicating whether the shutdown signal has been received
    shutdown: bool,

    // receive half of channel to listen for shutdown
    notify: broadcast::Receiver<()>,
}

impl Shutdown {
    pub fn new(notify: broadcast::Receiver<()>) -> Self {
        Self {
            shutdown: false,
            notify,
        }
    }

    pub fn is_shutdown(&self) -> bool {
        self.shutdown
    }

    pub async fn recv(&mut self) {
        // shutdown signal has already been received
        if self.shutdown {
            return;
        }

        // receive the shutdown signal
        let _ = self.notify.recv().await;
        self.shutdown = true;
    }
}
