use crate::server::LoginSession;
use slime_net::Packet;

/// Login server: register pic packet (0x1D)
/// TODO
pub async fn handle(mut packet: Packet, session: &mut LoginSession) -> anyhow::Result<()> {
    if packet.read_byte() == 0 {
        return session.stream.close().await;
    }

    let character_id = packet.read_int();
    let mac_addr = packet.read_string();
    let host_addr = packet.read_string();
    // TODO we can check mac_addr/hwid from host_addr if we want to prevent multi-logging

    let pic = packet.read_string();

    if pic.is_empty() {
        return session.stream.close().await;
    }

    // Set account's pic column
    sqlx::query("UPDATE accounts SET pic = ? WHERE id = ?")
        .bind(pic.clone())
        .bind(session.data.account_id)
        .execute(&session.db)
        .await?;

    session.data.pic = pic;
    session.data.character_id = character_id;

    super::select_character::connect_to_world_server(session).await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{config::Config, model::*};
    use slime_net::MapleStream;
    use sqlx::{MySql, Pool};
    use std::sync::Arc;
    use tokio::net::{TcpListener, TcpStream};

    #[sqlx::test(migrations = "../migrations")]
    async fn test(pool: Pool<MySql>) -> anyhow::Result<()> {
        dotenvy::dotenv().ok();

        let packet = Packet::empty();

        let listener = TcpListener::bind("0.0.0.0:8484").await?;
        let addr = listener.local_addr()?;
        let tcp_stream = TcpStream::connect(addr).await?;
        let maple_stream = MapleStream::new(tcp_stream);

        let mut login_session = LoginSession {
            id: 1,
            stream: maple_stream,
            db: pool,
            data: LoginSessionData::default(),
            config: Arc::new(Config::default()),
        };

        handle(packet, &mut login_session).await?;

        Ok(())
    }
}
