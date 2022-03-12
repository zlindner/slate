use super::{codec::MapleCodec, packet::Packet};
use crate::{
    crypto::cipher::{Cipher, CipherType},
    login::packets,
    Result,
};

use futures::SinkExt;
use tokio::{io::AsyncWriteExt, net::TcpStream};
use tokio_stream::StreamExt;
use tokio_util::codec::{Decoder, Framed};

#[derive(Debug)]
pub struct Connection {
    stream: Framed<TcpStream, MapleCodec>,
}

impl Connection {
    pub async fn new(mut socket: TcpStream) -> Result<Self> {
        let send = Cipher::new(CipherType::Send);
        let recv = Cipher::new(CipherType::Receive);

        Self::handshake(&mut socket, &send, &recv).await?;

        Ok(Self {
            stream: MapleCodec::new(send, recv).framed(socket),
        })
    }

    pub async fn read_packet(&mut self) -> Result<Option<Packet>> {
        loop {
            return Ok(self.stream.try_next().await?);
        }
    }

    pub async fn write_packet(&mut self, packet: Packet) -> Result<()> {
        self.stream.send(packet).await?;
        self.stream.flush().await?;
        Ok(())
    }

    pub async fn disconnect(&mut self) -> Result<()> {
        Ok(self.stream.close().await?)
    }

    // write the login server handshake packet directly to the TcpStream
    // we can't use write_packet() here since that will automatically encrypt the packet, and the
    // handshake is required to be unencrypted to setup client <-> server encryption
    async fn handshake(socket: &mut TcpStream, send: &Cipher, recv: &Cipher) -> Result<()> {
        let handshake = packets::handshake(&send, &recv);
        socket.write_all(&handshake.bytes).await?;
        socket.flush().await?;
        Ok(())
    }
}
