use crate::packet_handler;
use slime_data::sql;
use slime_net::MapleStream;
use sqlx::{MySql, Pool};
use std::time::Instant;
use tokio::net::TcpListener;

pub struct ChannelServer {
    pub ip: String,
    pub base_port: String,
    pub db: Pool<MySql>,
}

// TODO currently doesn't handle shutdown, so world will stay online even when channel is shutdown
impl ChannelServer {
    pub async fn start(self) -> anyhow::Result<()> {
        let (channel, addr) = self.get_available_channel().await?;
        let listener = TcpListener::bind(&addr).await?;
        log::info!("{} {} started @ {}", channel.world_name, channel.id, &addr);
        self.execute_startup_tasks(&channel).await?;

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
            };

            // Spawn a task for handling the new login session
            tokio::spawn(async move {
                Self::handle_session(session).await;
            });
        }
    }

    /// Gets the first available address to listen on
    async fn get_available_channel(&self) -> anyhow::Result<(sql::Channel, String)> {
        let channel =
            sqlx::query_as::<_, sql::Channel>("SELECT * FROM channels WHERE is_online = false")
                .fetch_one(&self.db)
                .await?;

        // TODO handle no channel being available

        let base_port: i32 = self
            .base_port
            .parse()
            .expect("Base port should be a valid integer");

        // ex. 10000 + ((1 - 1) * 1000) + (1 - 1) = 10000 (world 1, channel 1)
        // ex. 10000 + ((2 - 1) * 1000) + (2 - 1) = 11001 (world 2, channel 2)
        let port = base_port + ((channel.world_id - 1) * 1000) + (channel.id - 1);
        let addr = format!("{}:{}", self.ip, port);
        Ok((channel, addr))
    }

    /// Executes startup tasks
    async fn execute_startup_tasks(&self, channel: &sql::Channel) -> anyhow::Result<()> {
        let start = Instant::now();
        log::info!("Executing startup tasks...");

        sqlx::query("UPDATE channels SET is_online = 1 WHERE id = ? AND world_id = ?")
            .bind(channel.id)
            .bind(channel.world_id)
            .execute(&self.db)
            .await?;

        log::info!("Finished startup tasks in {:?}", start.elapsed());
        Ok(())
    }

    /// Handles a new channel session
    async fn handle_session(mut session: ChannelSession) {
        log::info!("Created channel session [id: {}]", session.id);

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

        log::info!("Channel session ended [id: {}]", session.id);
    }
}

pub struct ChannelSession {
    pub id: i32,
    pub stream: MapleStream,
    pub db: Pool<MySql>,
}
