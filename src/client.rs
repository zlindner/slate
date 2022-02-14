use crate::maple_aes::MapleAES;
use crate::maple_codec::MapleCodec;
use crate::{login, packet::Packet};

use deadpool_postgres::Pool;
use futures::SinkExt;
use std::error::Error;
use std::net::SocketAddr;
use tokio::net::TcpStream;
use tokio_stream::StreamExt;
use tokio_util::codec::{Decoder, Framed};

// TODO add session id?
pub struct Client {
    pub stream: Framed<TcpStream, MapleCodec>,
    addr: SocketAddr,
    pub pool: Pool,
    client_type: ClientType,
    // 0: receive, 1: send
    pub ciphers: (MapleAES, MapleAES),
}

#[derive(Debug, PartialEq)]
pub enum ClientType {
    Login,
    Channel,
    CashShop,
}

impl Client {
    pub fn new(stream: TcpStream, addr: SocketAddr, pool: Pool, client_type: ClientType) -> Self {
        log::info!("Client connected to {:?} server: {}", client_type, addr);

        let ciphers = init_ciphers();
        let stream = MapleCodec::new(ciphers.clone()).framed(stream);

        Client {
            stream,
            addr,
            pool,
            client_type,
            ciphers,
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
                        0x1 => login::handlers::login(packet, &mut self).await,
                        _ => log::warn!("Unhandled packet 0x{:X?}", op_code),
                    }
                }
                Err(e) => log::error!("Client disconnected with error: {:?}", e),
            }
        }

        log::info!("Client disconnected from {:?} server", self.client_type);

        Ok(())
    }

    pub async fn send_packet(&mut self, packet: Packet) -> Result<(), Box<dyn Error>> {
        self.stream.send(packet).await?;
        self.stream.flush().await?;

        Ok(())
    }
}

fn init_ciphers() -> (MapleAES, MapleAES) {
    let recv_iv: [u8; 4] = [0x46, 0x72, rand::random::<u8>(), 0x52];
    let recv_cipher = MapleAES::new(recv_iv, 83);

    let send_iv: [u8; 4] = [0x52, 0x30, 0x78, 0x61];
    let send_cipher = MapleAES::new(send_iv, 0xffff - 83);

    (recv_cipher, send_cipher)
}
