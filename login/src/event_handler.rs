use crate::{packet_handler::LoginServerPacketHandler, packets};
use async_trait::async_trait;
use bytes::BufMut;
use futures::SinkExt;
use oxide_core::net::{codec::MapleCodec, Events, Packet};
use tokio::net::TcpStream;
use tokio_util::codec::Framed;

pub struct LoginServerEventHandler;

#[async_trait]
impl Events for LoginServerEventHandler {
    async fn on_start(&self, addr: &str) {
        log::info!("Login server started @ {}", addr);
    }

    async fn on_connect(&self, stream: &mut Framed<TcpStream, MapleCodec>) {
        log::info!("Client connected to login server");

        let codec = stream.codec();
        let handshake = packets::handshake(&codec.send, &codec.recv);
        stream.write_buffer_mut().put_slice(&handshake.bytes);

        if let Err(e) = stream.flush().await {
            log::error!("Error writing handshake packet: {}", e);
        }
    }

    async fn on_packet(&self, stream: &mut Framed<TcpStream, MapleCodec>, packet: Packet) {
        log::debug!("Received packet: {}", packet);

        if let Err(e) = LoginServerPacketHandler::get(packet).handle(stream).await {
            log::error!("Handle packet error: {}", e);
        }
    }

    async fn on_disconnect(&self, stream: &mut Framed<TcpStream, MapleCodec>) {
        log::info!("Client disconnected from login server");
    }
}
