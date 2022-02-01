mod maple_aes;
use crate::maple_aes::maple_aes::MapleAES;

mod packet;
use crate::packet::packet::Packet;

mod shanda;
use crate::shanda::shanda::decrypt;

use simple_logger::SimpleLogger;
use std::error::Error;
use std::net::SocketAddr;
use tokio::{
    io::AsyncWriteExt,
    net::{TcpListener, TcpStream},
};
use tokio_stream::StreamExt;
use tokio_util::codec::{BytesCodec, Decoder};

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

    let iv_receive: [u8; 4] = [0x46, 0x72, rand::random::<u8>(), 0x52];
    log::debug!("iv_receive: {:#04X?}", iv_receive);
    let mut cypher_receive = MapleAES::new(iv_receive, 0xffff - 83);

    let iv_send: [u8; 4] = [0x52, 0x30, 0x78, 0x61];
    log::debug!("iv_send: {:#04X?}", iv_send);
    //let cypher_send = MapleAES::new(iv_receive, 83);

    // write the initial unencrypted "hello" packet
    let packet = login_handshake(iv_receive, iv_send);
    stream.write_all(&packet.bytes()).await?;
    stream.flush().await?;

    let mut framed = BytesCodec::new().framed(stream);

    while let Some(message) = framed.next().await {
        match message {
            Ok(bytes) => {
                let bytes = bytes.to_vec();
                log::debug!("receieved: {:#04X?}", bytes);

                // TODO validate the packet header
                log::debug!("packet header: {:#04X?}", &bytes[0..3]);
                log::debug!("packet data: {:#04X?}", &bytes[4..]);

                let transformed = cypher_receive.transform((&bytes[4..]).to_vec());
                log::debug!("transformed: {:#04X?}", transformed);

                let decrypted = decrypt(transformed);
                log::debug!("decrypted: {:#04X?}", decrypted);

                let op_code = u16::from_le_bytes([decrypted[0], decrypted[1]]);
                log::debug!("op_code: {}", op_code);
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
