mod maple_aes;
use crate::maple_aes::MapleAES;

mod packet;
use crate::packet::Packet;

mod shanda;

mod maple_codec;
use crate::maple_codec::MapleCodec;

use simple_logger::SimpleLogger;
use std::error::Error;
use std::net::SocketAddr;
use tokio::{
    io::AsyncWriteExt,
    net::{TcpListener, TcpStream},
};
use tokio_stream::StreamExt;
use tokio_util::codec::Decoder;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    SimpleLogger::new().env().init().unwrap();

    let listener = TcpListener::bind("127.0.0.1:8484").await?;
    log::info!("Login server started on port 8484");

    loop {
        let (stream, addr) = listener.accept().await?;

        tokio::spawn(async move {
            if let Err(e) = handle_connection(stream, addr).await {
                log::error!("An error occurred: {:?}", e);
            }
        });
    }
}

async fn handle_connection(mut stream: TcpStream, addr: SocketAddr) -> Result<(), Box<dyn Error>> {
    log::info!("Client connected to login server: {}", addr);

    let recv_iv: [u8; 4] = [0x46, 0x72, rand::random::<u8>(), 0x52];
    log::debug!("recv_iv: {:X?}", recv_iv);
    let recv_cipher = MapleAES::new(recv_iv, 83);

    let send_iv: [u8; 4] = [0x52, 0x30, 0x78, 0x61];
    log::debug!("send_iv: {:X?}", send_iv);
    let send_cipher = MapleAES::new(send_iv, 0xffff - 83);

    // write the initial unencrypted "hello" packet
    let handshake = login_handshake(recv_iv, send_iv);
    stream.write_all(&handshake.get_data()).await?;
    stream.flush().await?;

    let mut framed = MapleCodec::new(recv_cipher, send_cipher).framed(stream);

    while let Some(message) = framed.next().await {
        match message {
            Ok(mut packet) => {
                log::debug!("received packet: {}", packet);

                let op_code = packet.read_short();
                log::debug!("op_code: {}", op_code);
                // TODO get the packet handler for the op_code
            }
            Err(err) => println!("Socket closed with error: {:?}", err),
        }
    }

    println!("Socket received FIN packet and closed connection");

    Ok(())
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
