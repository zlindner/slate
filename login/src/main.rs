use async_trait::async_trait;
use log::LevelFilter;
use oxide_core::{
    db,
    net::{codec::MapleCodec, Events, Packet, Server},
    Result,
};
use simple_logger::SimpleLogger;
use std::env;
use tokio::net::TcpStream;
use tokio_util::codec::Framed;

mod handler;

struct LoginServerEventHandler;

#[async_trait]
impl Events for LoginServerEventHandler {
    async fn on_start(&self, addr: &str) {
        log::info!("Login server started @ {}", addr);
    }

    async fn on_connect(&self, stream: &mut Framed<TcpStream, MapleCodec>) {
        log::info!("Client connected to login server");
        // TODO think we should be able to get the raw tcpstream here to write the unencrypted handshake packet
    }

    async fn on_packet(&self, stream: &mut Framed<TcpStream, MapleCodec>, packet: Packet) {
        log::debug!("Received packet: {}", packet);
    }

    async fn on_disconnect(&self, stream: &mut Framed<TcpStream, MapleCodec>) {
        log::info!("Client disconnected from login server");
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    dotenv::dotenv().ok();

    SimpleLogger::new()
        .with_module_level("tokio_util", LevelFilter::Debug)
        .with_module_level("mio", LevelFilter::Debug)
        .env()
        .init()
        .unwrap();

    let db = db::new(10).await?;

    let x = Server::new(
        env::var("LOGIN_SERVER_ADDR").unwrap(),
        LoginServerEventHandler,
    );

    x.start().await?;

    Ok(())
}
