use crate::maple_aes::MapleAES;
use crate::packet::Packet;
use crate::shanda::Shanda;

use bytes::{BufMut, Bytes, BytesMut};
use std::io;
use tokio_util::codec::{Decoder, Encoder};

pub struct MapleCodec {
    recv_cipher: MapleAES,
    send_cipher: MapleAES,
}

impl MapleCodec {
    pub fn new(recv_cipher: MapleAES, send_cipher: MapleAES) -> Self {
        MapleCodec {
            recv_cipher,
            send_cipher,
        }
    }
}

impl Decoder for MapleCodec {
    type Item = Packet;
    type Error = io::Error;

    fn decode(&mut self, buf: &mut BytesMut) -> Result<Option<Packet>, io::Error> {
        if buf.is_empty() {
            return Ok(None);
        }

        let len = buf.len();
        let mut bytes = buf.split_to(len);

        // header: [0, 4), body: [4, len)
        let body = bytes.split_off(4);
        let header = bytes;

        // validate the packet header
        if !self.recv_cipher.is_valid_header(&header) {
            log::error!("receieved a packet with an invalid header: {:#x}", header);

            return Ok(None);
        }

        // transform and decrypt the incoming packet's body
        let decrypted = Shanda::decrypt(self.recv_cipher.transform(body));

        Ok(Some(Packet::from_bytes(decrypted)))
    }
}

// TODO
impl Encoder<Bytes> for MapleCodec {
    type Error = io::Error;

    fn encode(&mut self, data: Bytes, buf: &mut BytesMut) -> Result<(), io::Error> {
        buf.reserve(data.len());
        buf.put(data);
        Ok(())
    }
}
