use deadpool_postgres::Pool;
use std::error::Error;
use std::net::SocketAddr;
use tokio::{io::AsyncWriteExt, net::TcpStream};
use tokio_stream::StreamExt;
use tokio_util::codec::Decoder;

use crate::maple_aes::MapleAES;
use crate::maple_codec::MapleCodec;
use crate::packet::Packet;

// TODO add session id?
pub struct Client {
    stream: TcpStream,
    addr: SocketAddr,
    pool: Pool,
    client_type: ClientType,
    recv_cipher: MapleAES,
    send_cipher: MapleAES,
}

#[derive(Debug)]
pub enum ClientType {
    LOGIN,
    CHANNEl,
}

impl Client {
    pub fn new(stream: TcpStream, addr: SocketAddr, pool: Pool, client_type: ClientType) -> Self {
        log::info!("Client connected to {:?} server: {}", client_type, addr);

        let recv_iv: [u8; 4] = [0x46, 0x72, rand::random::<u8>(), 0x52];
        let recv_cipher = MapleAES::new(recv_iv, 83);

        let send_iv: [u8; 4] = [0x52, 0x30, 0x78, 0x61];
        let send_cipher = MapleAES::new(send_iv, 0xffff - 83);

        Client {
            stream,
            addr,
            pool,
            client_type,
            recv_cipher,
            send_cipher,
        }
    }

    pub async fn handle_packets(mut self) -> Result<(), Box<dyn Error>> {
        // TODO check self.client_type, add handler function for login/channel client

        // write the initial unencrypted "hello" packet
        let handshake = login_handshake(self.recv_cipher.iv, self.send_cipher.iv);
        self.stream.write_all(&handshake.get_data()).await?;
        self.stream.flush().await?;

        let mut framed = MapleCodec::new(self.recv_cipher, self.send_cipher).framed(self.stream);

        while let Some(message) = framed.next().await {
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
                        0x1 => handle_login_password(packet, &self.pool).await,
                        _ => log::warn!("Unhandled packet 0x{:X?}", op_code),
                    }
                }
                Err(e) => log::error!("Client disconnected with error: {:?}", e),
            }
        }

        log::info!("Client disconnected from {:?} server", self.client_type);

        Ok(())
    }
}

fn login_handshake(iv_receive: [u8; 4], iv_send: [u8; 4]) -> Packet {
    let mut packet = Packet::new(18);
    packet.write_short(14); // packet length (0x0E)
    packet.write_short(83); // maple version (v83)
    packet.write_maple_string("1"); // maple patch version (1)
    packet.write_bytes(&iv_receive);
    packet.write_bytes(&iv_send);
    packet.write_byte(8); // locale
    packet
}

// TODO need to look into whether having async db is actually better
async fn handle_login_password(mut packet: Packet, pool: &Pool) {
    let username = packet.read_maple_string();
    log::debug!("username: {}", username);

    let password = packet.read_maple_string();
    log::debug!("password: {}", password);

    packet.advance(6);

    let hwid = packet.read_bytes(4);
    log::debug!("hwid: {:02X?}", hwid);

    let client = pool.get().await.unwrap();
    let rows = client
        .query(
            "SELECT password FROM accounts WHERE name = $1",
            &[&username],
        )
        .await
        .unwrap();

    if rows.len() == 0 {
        log::debug!("Account doesn't exist");
    }
}
