use anyhow::Result;
use bytes::BytesMut;
use client::CygnusClient;
use handler::PacketHandler;
use log::LevelFilter;
use oxy_core::{crypt::MapleAES, net::Packet};
use rand::random;
use simple_logger::SimpleLogger;
use tokio::{io::AsyncReadExt, net::TcpStream};

mod client;
mod handler;

#[tokio::main]
async fn main() -> Result<()> {
    SimpleLogger::new()
        .with_module_level("mio", LevelFilter::Off)
        .env()
        .init()
        .unwrap();

    let mut stream = TcpStream::connect("0.0.0.0:8484").await?;

    let mut handshake = BytesMut::zeroed(16);
    stream.read_exact(&mut handshake).await?;

    let mut handshake = Packet::wrap(handshake);
    handshake.skip(2);
    let version = handshake.read_short() as u16;
    handshake.skip(3);
    let recv_iv: [u8; 4] = (*handshake.read_bytes(4)).try_into()?;
    let send_iv: [u8; 4] = (*handshake.read_bytes(4)).try_into()?;

    let aes = MapleAES::new_with_iv(version, recv_iv, send_iv);
    let mut client = CygnusClient::new(stream, aes);

    // send login_start
    let packet = login_start();
    client.send(packet).await?;

    // send login
    let packet = login("cygnus", "test1234");
    client.send(packet).await?;

    loop {
        let packet = match client.read().await {
            Ok(packet) => packet,
            Err(e) => {
                log::error!("Error reading packet: {}", e);
                break;
            }
        };

        if let Err(e) = PacketHandler::handle(packet, &mut client).await {
            log::error!("Error handling packet: {}", e);
        }
    }

    Ok(())
}

/// Packet indicating the client has hit the login screen
fn login_start() -> Packet {
    let mut packet = Packet::new();
    packet.write_short(0x23);
    packet
}

///
fn login(name: &str, password: &str) -> Packet {
    let mut packet = Packet::new();
    packet.write_short(0x01);
    packet.write_string(name);
    packet.write_string(password);
    packet.write_bytes(&[0u8; 6]);
    packet.write_bytes(&random::<[u8; 4]>());
    packet
}

#[tokio::test]
async fn test_handshake() -> Result<()> {
    let mut stream = TcpStream::connect("0.0.0.0:8484").await?;

    let mut handshake = BytesMut::zeroed(16);
    stream.read_exact(&mut handshake).await?;

    let mut handshake = Packet::wrap(handshake);
    log::debug!("Handshake: {}", handshake);

    let op = handshake.read_short();
    assert_eq!(0x0E, op);
    let version = handshake.read_short();
    assert_eq!(83, version);
    let patch = handshake.read_string();
    assert_eq!("1", patch);
    let _recv_iv = handshake.read_bytes(4);
    let _send_iv = handshake.read_bytes(4);
    let locale = handshake.read_byte();
    assert_eq!(8, locale);

    Ok(())
}
