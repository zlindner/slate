use crate::crypto::maple_aes::MapleAES;
use crate::login::handlers::Account;
use crate::net::{maple_codec::MapleCodec, packet::Packet};
use crate::{login, Server};

use deadpool_postgres::Pool;
use futures::SinkExt;
use std::error::Error;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::TcpStream;
use tokio::sync::Mutex;
use tokio_stream::StreamExt;
use tokio_util::codec::{Decoder, Framed};

// TODO add session id?
pub struct Client {
    pub stream: Framed<TcpStream, MapleCodec>,
    addr: SocketAddr,
    pub server: Arc<Mutex<Server>>,
    pub pool: Pool,
    client_type: ClientType,
    // 0: receive, 1: send
    pub ciphers: (MapleAES, MapleAES),
    pub account: Option<Account>,
}

#[derive(Debug, PartialEq)]
pub enum ClientType {
    Login,
    Channel,
    CashShop,
}

#[derive(PartialEq, Clone, Copy, Debug)]
pub enum LoginState {
    LoggedOut = 0,
    Transitioning = 1,
    LoggedIn = 2,
    Error = -1,
}

impl Client {
    pub fn new(
        stream: TcpStream,
        addr: SocketAddr,
        server: Arc<Mutex<Server>>,
        pool: Pool,
        client_type: ClientType,
    ) -> Self {
        log::info!("Client connected to {:?} server: {}", client_type, addr);

        let ciphers = init_ciphers();
        let stream = MapleCodec::new(ciphers.clone()).framed(stream);

        Client {
            stream,
            addr,
            server,
            pool,
            client_type,
            ciphers,
            account: None,
        }
    }

    pub async fn connect(mut self) -> Result<(), Box<dyn Error>> {
        // TODO check self.client_type, add handler function for login/channel client

        if self.client_type == ClientType::Login {
            let handshake = login::packets::handshake(&self.ciphers);
            self.stream.send(handshake).await?;
            self.stream.flush().await?;
        }

        while let Some(message) = self.stream.next().await {
            match message {
                Ok(mut packet) => {
                    log::debug!("received packet: {}", packet);

                    let op_code = packet.read_short();

                    if op_code >= 0x200 {
                        log::warn!(
                            "Potential malicious packet sent to {:?} server from {}: 0x{:X?}",
                            self.client_type,
                            self.addr,
                            op_code
                        );

                        break;
                    }

                    match op_code {
                        0x01 => login::handlers::login(packet, &mut self).await,
                        0x05 => login::handlers::character_list(packet, &mut self).await,
                        0x06 => login::handlers::world_status(packet, &mut self).await,
                        0x07 => login::handlers::accept_tos(packet, &mut self).await,
                        0x0B => login::handlers::world_list(packet, &mut self).await,
                        _ => log::warn!("Unhandled packet 0x{:X?}", op_code),
                    }
                }
                Err(e) => log::error!("Client disconnected with error: {:?}", e),
            }
        }

        // socket received the FIN packet/disconnect() was called
        self.on_disconnect().await;

        Ok(())
    }

    pub async fn disconnect(&mut self) {
        // close the client's socket, if successful on_disconnect() will be called from connect()
        if let Err(e) = self.stream.close().await {
            log::error!("An error occurred while disconnecting: {}", e);
        }
    }

    async fn on_disconnect(&mut self) {
        log::info!("Client disconnected from {:?} server", self.client_type);

        self.update_login_state(LoginState::LoggedOut).await;
    }

    pub async fn send_packet(&mut self, packet: Packet) {
        if let Err(e) = self.stream.send(packet).await {
            log::debug!("An error occurred while sending packet: {}", e);
        }

        if let Err(e) = self.stream.flush().await {
            log::debug!("An error occurred while flusing stream: {}", e);
        }
    }

    pub async fn update_login_state(&mut self, new_state: LoginState) {
        if self.account.is_none() {
            log::error!("Client's account is None");
            return;
        }

        let account: &mut Account = self.account.as_mut().unwrap();
        account.login_state = new_state;

        let db = self.pool.get().await.unwrap();

        if let Err(e) = db
            .query(
                "UPDATE accounts SET login_state = $1, last_login = CURRENT_TIMESTAMP WHERE id = $2",
                &[&(new_state as i16), &account.id],
            )
            .await
        {
            log::error!("An error occurred while updating login state: {}", e);
            return;
        }
    }
}

fn init_ciphers() -> (MapleAES, MapleAES) {
    let recv_iv: [u8; 4] = [0x46, 0x72, rand::random::<u8>(), 0x52];
    let recv_cipher = MapleAES::new(recv_iv, 83);

    let send_iv: [u8; 4] = [0x52, 0x30, 0x78, 0x61];
    let send_cipher = MapleAES::new(send_iv, 0xffff - 83);

    (recv_cipher, send_cipher)
}
