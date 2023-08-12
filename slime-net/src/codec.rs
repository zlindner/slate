use crate::{MapleAES, Packet};
use anyhow::{anyhow, Result};
use bytes::{BufMut, BytesMut};
use tokio_util::codec::{Decoder, Encoder};

pub struct MapleCodec {
    pub aes: MapleAES,
}

impl MapleCodec {
    pub fn new() -> Self {
        Self {
            aes: MapleAES::new(83),
        }
    }
}

impl Decoder for MapleCodec {
    type Item = Packet;
    type Error = anyhow::Error;

    fn decode(&mut self, buf: &mut BytesMut) -> Result<Option<Packet>> {
        // header (4) + op code (2)
        if buf.len() < 6 {
            return Ok(None);
        }

        let header = buf.split_to(4);

        if !self.aes.is_valid_header(&header) {
            return Err(anyhow!("Invalid packet header: {:02X?}", header));
        }

        let len = self.aes.get_packet_len(&header);

        if len as usize > buf.len() {
            log::warn!(
                "Packet length {} is greater than buf length {}",
                len,
                buf.len()
            );
        }

        let mut body = buf.split_to(len as usize);
        self.aes.decrypt(&mut body);
        Ok(Some(Packet::wrap(body)))
    }
}

impl Encoder<Packet> for MapleCodec {
    type Error = anyhow::Error;

    fn encode(&mut self, mut packet: Packet, buf: &mut BytesMut) -> Result<()> {
        if packet.use_encryption {
            let header = self.aes.generate_header(packet.len());
            self.aes.encrypt(&mut packet.bytes);
            buf.reserve(packet.len() + header.len());
            buf.put(header.as_slice());
        } else {
            buf.reserve(packet.len());
        }

        buf.put(packet.bytes);
        Ok(())
    }
}
