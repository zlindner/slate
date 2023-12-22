use tokio::sync::broadcast;

pub struct Shutdown {
    /// Whether the shutdown signal has been received
    is_shutdown: bool,

    /// The receive half of the channel used to listen for shutdown
    notify: broadcast::Receiver<()>,
}

impl Shutdown {
    pub fn new(notify: broadcast::Receiver<()>) -> Shutdown {
        Shutdown {
            is_shutdown: false,
            notify,
        }
    }

    /// Returns true if the shutdown signal has been received
    pub fn is_shutdown(&self) -> bool {
        self.is_shutdown
    }

    /// Receive the shutdown notice, waiting if necessary
    pub async fn recv(&mut self) {
        // If the shutdown signal has already been received, then return
        // immediately
        if self.is_shutdown {
            return;
        }

        // Cannot receive a "lag error" as only one value is ever sent
        let _ = self.notify.recv().await;

        // Remember that the signal has been received
        self.is_shutdown = true;
    }
}
