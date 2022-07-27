use futures::SinkExt;
use oxide_core::{
    net::{cipher::Cipher, codec::MapleCodec, Packet},
    Result,
};
use tokio::{io::AsyncWriteExt, net::TcpStream};
use tokio_stream::StreamExt;
use tokio_util::codec::{Decoder, Framed};

use crate::login::packets;

pub struct Connection {
    stream: Framed<TcpStream, MapleCodec>,
}

impl Connection {
    pub async fn new(mut socket: TcpStream) -> Result<Self> {
        let send = Cipher::new(0xffff - 83);
        let recv = Cipher::new(83);

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
        // FIXME not sure if we actually need to flush on every packet write?
        self.stream.flush().await?;
        Ok(())
    }

    pub async fn disconnect(&mut self) -> Result<()> {
        Ok(self.stream.close().await?)
    }

    // FIXME: this should be in login server rather than connection
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
