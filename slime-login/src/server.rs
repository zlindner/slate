use crate::packet_handler;
use slime_data::{
    sql::{self, account::LoginState},
    Config,
};
use slime_net::MapleStream;
use sqlx::{MySql, Pool};
use std::{sync::Arc, time::Instant};
use tokio::net::TcpListener;

pub struct LoginServer {
    pub addr: String,
    pub db: Pool<MySql>,
    pub config: Arc<Config>,
}

impl LoginServer {
    /// Starts the login server
    pub async fn start(self) -> anyhow::Result<()> {
        let listener = TcpListener::bind(&self.addr).await?;
        log::info!("Login server started @ {}", &self.addr);
        self.execute_startup_tasks().await?;

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
                data: sql::LoginSession::default(),
                config: self.config.clone(),
                transitioning: false,
            };

            // Spawn a task for handling the new login session
            tokio::spawn(async move {
                Self::handle_session(session).await;
            });
        }
    }

    /// Execute startup tasks
    async fn execute_startup_tasks(&self) -> anyhow::Result<()> {
        let start = Instant::now();
        log::info!("Executing startup tasks...");

        sqlx::query("UPDATE accounts SET state = ?")
            .bind(LoginState::LoggedOut)
            .execute(&self.db)
            .await?;

        sqlx::query("DELETE FROM login_sessions")
            .execute(&self.db)
            .await?;

        log::info!("Finished startup tasks in {:?}", start.elapsed());
        Ok(())
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

        // If we aren't transitioning to the channel server, set the account's state to `LoggedOut`
        if !session.transitioning {
            sql::Account::update_login_state(
                session.data.account_id,
                LoginState::LoggedOut,
                &session.db,
            )
            .await
            .unwrap();
        }
    }
}

pub struct LoginSession {
    pub id: i32,
    pub stream: MapleStream,
    pub db: Pool<MySql>,
    pub data: sql::LoginSession,
    pub config: Arc<Config>,

    // Indicates we are transitioning to the channel server
    pub transitioning: bool,
}
