use crate::{config::Config, model::LoginSessionData, packet_handler};
use anyhow::Result;
use slime_net::MapleStream;
use sqlx::{MySql, Pool};
use std::sync::Arc;
use tokio::net::TcpListener;

pub struct LoginServer {
    pub addr: String,
    pub db: Pool<MySql>,
    pub config: Arc<Config>,
}

impl LoginServer {
    /// Starts the login server
    pub async fn start(self) -> Result<()> {
        let listener = TcpListener::bind(&self.addr).await?;
        log::info!("Login server started @ {}", &self.addr);

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

            let session = LoginSession {
                id: session_id,
                stream,
                db: self.db.clone(),
                data: LoginSessionData::default(),
                config: self.config.clone(),
            };

            // Spawn a task for handling the new login session
            tokio::spawn(async move {
                Self::handle_session(session).await;
            });
        }
    }

    /// Handles a new login session
    async fn handle_session(mut session: LoginSession) {
        log::info!("Created login session [id: {}]", session.id);

        // Send unencrypted handshake to client -- sets up encryption IVs
        if let Err(e) = session.stream.write_handshake().await {
            log::error!("Handshake error: {} [id: {}]", e, session.id);
            return;
        }

        // Keep reading packets from the client in a loop until they disconnect
        // or an error occurs
        loop {
            let packet = match session.stream.read_packet().await {
                Some(Ok(packet)) => packet,
                Some(Err(e)) => {
                    log::error!("Error reading packet: {} [id: {}]", e, session.id);
                    break;
                }
                // Client disconnected/sent EOF
                None => break,
            };

            if let Err(e) = packet_handler::handle_packet(packet, &mut session).await {
                log::error!("Error handling packet: {} [id: {}]", e, session.id);
            }
        }

        log::info!("Login session ended [id: {}]", session.id);
    }
}

pub struct LoginSession {
    pub id: i32,
    pub stream: MapleStream,
    pub db: Pool<MySql>,
    pub data: LoginSessionData,
    pub config: Arc<Config>,
}
