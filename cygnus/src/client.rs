use anyhow::{anyhow, Result};
use bytes::BytesMut;
use oxy_core::{crypt::MapleAES, net::Packet, prisma::session};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
};

pub struct CygnusClient {
    stream: TcpStream,
    aes: MapleAES,
    pub session: session::Data,
}

impl CygnusClient {
    pub fn new(stream: TcpStream, aes: MapleAES) -> Self {
        let session = session::Data {
            id: 1,
            account_id: -1,
            character_id: -1,
            world_id: -1,
            chanel_id: -1,
            login_attempts: 0,
            pin: "".to_string(),
            pin_attempts: 0,
            pic: "".to_string(),
            pic_attempts: 0,
        };

        Self {
            stream,
            aes,
            session,
        }
    }

    pub async fn send(&mut self, mut packet: Packet) -> Result<()> {
        log::debug!("Sent: {}", packet);
        let header = self.aes.build_header_cygnus(packet.len());
        self.aes.encrypt(&mut packet.bytes);
        self.stream.write_all(&header).await?;
        self.stream.write_all(&packet.bytes).await?;
        Ok(())
    }

    pub async fn read(&mut self) -> Result<Packet> {
        let mut header = [0u8; 4];
        self.stream.read_exact(&mut header).await?;

        if !self.aes.is_valid_header_cygnus(&header) {
            return Err(anyhow!("Invalid packet header: {:02X?}", header));
        }

        let len = self.aes.get_packet_len(&header);
        let mut body = BytesMut::zeroed(len as usize);
        self.stream.read_exact(&mut body).await?;
        self.aes.decrypt(&mut body);
        Ok(Packet::wrap(body))
    }
}
