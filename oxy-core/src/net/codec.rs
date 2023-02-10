use super::Packet;
use crate::crypt::MapleAES;
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

        if !self.aes.is_valid_header(&buf) {
            return Err(anyhow!("Invalid packet header"));
        }

        let len = self.aes.get_packet_len(&buf) as usize;

        // The current frame doesn't contain the entire packet
        if buf.len() < len + 4 {
            // TODO should remove this log once verified that this stuff works :)
            log::warn!(
                "Packet length {} is greater than buf length {}",
                len,
                buf.len()
            );
            return Ok(None);
        }

        // Remove packet header
        _ = buf.split_to(4);

        let mut body = buf.split_to(len);
        self.aes.decrypt(&mut body);
        Ok(Some(Packet::wrap(body)))
    }
}

impl Encoder<Packet> for MapleCodec {
    type Error = anyhow::Error;

    fn encode(&mut self, mut packet: Packet, buf: &mut BytesMut) -> Result<()> {
        if packet.use_encryption {
            let header = self.aes.build_header(packet.len());
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
