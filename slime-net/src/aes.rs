use crate::Packet;
use aes::{
    cipher::{generic_array::GenericArray, BlockEncrypt, KeyInit},
    Aes256,
};
use anyhow::{anyhow, Result};
use bytes::{BufMut, BytesMut};
use rand::random;
use tokio_util::codec::{Decoder, Encoder};

/// Custom AES Maplestory encryption implementation
pub struct MapleAES {
    cipher: Aes256,
    send_iv: [u8; 4],
    recv_iv: [u8; 4],
    version: u16,
}

impl MapleAES {
    /// Creates an instance of MapleAES for the given version
    pub fn new(version: u16) -> Self {
        let mut key = super::KEY;
        let key = GenericArray::from_mut_slice(&mut key);

        Self {
            cipher: Aes256::new(key),
            send_iv: random::<[u8; 4]>(),
            recv_iv: random::<[u8; 4]>(),
            version: ((version >> 8) & 0xFF) | ((version << 8) & 0xFF00),
        }
    }

    /// Encrypts the given data
    pub fn encrypt(&mut self, data: &mut [u8]) {
        super::shanda::encrypt(data);
        self.send_iv = self.transform(data, self.send_iv);
    }

    /// Decrypts the given data
    pub fn decrypt(&mut self, data: &mut [u8]) {
        self.recv_iv = self.transform(data, self.recv_iv);
        super::shanda::decrypt(data);
    }

    /// Creates the handshake packet for sharing IVs with the client
    pub fn get_handshake(&self) -> Packet {
        let mut handshake = Packet::new(0x0E);
        handshake.write_short(83); // maple version
        handshake.write_string("1"); // maple patch version
        handshake.write_bytes(&self.recv_iv);
        handshake.write_bytes(&self.send_iv);
        handshake.write_byte(8); // locale
        handshake
    }

    /// Checks if the given header is valid
    pub fn is_valid_header(&self, header: &[u8]) -> bool {
        if header.len() != 4 {
            return false;
        }

        (header[0] ^ self.recv_iv[2]) == (self.version >> 8) as u8
            && ((header[1] ^ self.recv_iv[3]) == (self.version & 0xFF) as u8)
    }

    /// Generates a packet header with the given length
    pub fn generate_header(&self, len: usize) -> [u8; 4] {
        let mut iiv: u32 = self.send_iv[3] as u32 & 0xFF;
        iiv |= ((self.send_iv[2] as u32) << 8) & 0xFF00;
        iiv ^= 0xFFFF - self.version as u32;
        let mlength = (((len as u32) << 8) & 0xFF00) | ((len as u32) >> 8);
        let xored_iv = iiv ^ mlength;

        [
            (iiv >> 8) as u8,
            iiv as u8,
            (xored_iv >> 8) as u8,
            xored_iv as u8,
        ]
    }

    /// Gets the packet length from the given header
    pub fn get_packet_len(&self, header: &[u8]) -> i16 {
        (header[0] as i16 + ((header[1] as i16) << 8))
            ^ (header[2] as i16 + ((header[3] as i16) << 8))
    }

    /// Transforms data with the passed iv and returns a new iv
    fn transform(&self, data: &mut [u8], iv: [u8; 4]) -> [u8; 4] {
        let mut remaining = data.len();
        let mut start = 0;
        let mut block_length = 0x5B0;

        while remaining > 0 {
            let mut iv = iv.repeat(4);
            let iv = GenericArray::from_mut_slice(&mut iv);

            if remaining < block_length {
                block_length = remaining;
            }

            for i in start..(start + block_length) {
                if (i - start) % iv.len() == 0 {
                    self.cipher.encrypt_block(iv);
                }

                data[i] ^= iv[(i - start) % iv.len()];
            }

            start += block_length;
            remaining -= block_length;
            block_length = 0x5B4;
        }

        self.get_new_iv(iv)
    }

    /// Generates a new initialization vector
    fn get_new_iv(&self, iv: [u8; 4]) -> [u8; 4] {
        let mut new_iv = super::DEFAULT_IV;
        let shift_bytes = super::SHIFT_BYTES;

        (0..4).for_each(|i| {
            let byte = iv[i];
            new_iv[0] = new_iv[0].wrapping_add(shift_bytes[new_iv[1] as usize].wrapping_sub(byte));
            new_iv[1] = new_iv[1].wrapping_sub(new_iv[2] ^ shift_bytes[byte as usize]);
            new_iv[2] ^= shift_bytes[new_iv[3] as usize].wrapping_add(byte);
            new_iv[3] = new_iv[3].wrapping_add(shift_bytes[byte as usize].wrapping_sub(new_iv[0]));

            let mut mask = 0usize;
            mask |= (new_iv[0] as usize) & 0xFF;
            mask |= ((new_iv[1] as usize) << 8) & 0xFF00;
            mask |= ((new_iv[2] as usize) << 16) & 0xFF0000;
            mask |= ((new_iv[3] as usize) << 24) & 0xFF000000;
            mask = (mask >> 0x1D) | (mask << 3);

            (0..4).for_each(|j| {
                new_iv[j] = ((mask >> (8 * j)) & 0xFF) as u8;
            });
        });

        new_iv
    }
}

impl Decoder for MapleAES {
    type Item = Packet;
    type Error = anyhow::Error;

    fn decode(&mut self, buf: &mut BytesMut) -> Result<Option<Packet>> {
        // header (4) + op code (2)
        if buf.len() < 6 {
            return Ok(None);
        }

        // TODO do we need to split, why not just pass in entire packet?
        let header = buf.split_to(4);

        if !self.is_valid_header(&header) {
            return Err(anyhow!("Invalid packet header {:02X?}", header));
        }

        let len = self.get_packet_len(&header);

        if len as usize > buf.len() {
            log::warn!(
                "Packet length {} is greater than buf length {}",
                len,
                buf.len()
            );
        }

        let mut body = buf.split_to(len as usize);
        self.decrypt(&mut body);
        Ok(Some(Packet::wrap(body)))
    }
}

impl Encoder<Packet> for MapleAES {
    type Error = anyhow::Error;

    fn encode(&mut self, mut packet: Packet, buf: &mut BytesMut) -> Result<()> {
        if packet.use_encryption {
            let header = self.generate_header(packet.len());
            self.encrypt(&mut packet.bytes);
            buf.reserve(packet.len() + header.len());
            buf.put(header.as_slice());
        } else {
            buf.reserve(packet.len());
        }

        buf.put(packet.bytes);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Tests encryption and decryption of a reandom 16 byte packet:
    /// client -> server then server -> client
    #[test]
    fn encrypt_decrypt_test() {
        let mut server_aes = MapleAES::new(83);
        let mut handshake = server_aes.get_handshake();
        handshake.skip(7);

        let mut client_aes = MapleAES::new(83);
        client_aes.send_iv = (*handshake.read_bytes(4)).try_into().unwrap();
        client_aes.recv_iv = (*handshake.read_bytes(4)).try_into().unwrap();

        let mut packet = Packet::empty();
        let random_bytes = random::<[u8; 16]>();
        packet.write_bytes(&random_bytes);

        client_aes.encrypt(&mut packet.bytes);
        server_aes.decrypt(&mut packet.bytes);

        assert!(random_bytes.iter().eq(packet.bytes.iter()));

        let mut packet = Packet::empty();
        let random_bytes = random::<[u8; 16]>();
        packet.write_bytes(&random_bytes);

        server_aes.encrypt(&mut packet.bytes);
        client_aes.decrypt(&mut packet.bytes);

        assert!(random_bytes.iter().eq(packet.bytes.iter()));
    }

    /// Tests server validation of a client generated header
    #[test]
    fn server_validate_client_header() {
        let server_aes = MapleAES::new(83);
        let mut handshake = server_aes.get_handshake();
        handshake.skip(7);

        let mut client_aes = MapleAES::new(83);
        // client send_iv should be server recv_iv
        client_aes.send_iv = (*handshake.read_bytes(4)).try_into().unwrap();

        let header = generate_header_client(&client_aes, 8);
        assert!(server_aes.is_valid_header(&header));
        assert_eq!(server_aes.get_packet_len(&header), 8);
    }

    /// Tests client validation of a server generated header
    #[test]
    fn client_validate_server_header() {
        let server_aes = MapleAES::new(83);
        let mut handshake = server_aes.get_handshake();
        handshake.skip(11);

        let mut client_aes = MapleAES::new(83);
        // client recv_iv should be server send_iv
        client_aes.recv_iv = (*handshake.read_bytes(4)).try_into().unwrap();

        let header = server_aes.generate_header(8);
        assert!(is_valid_header_client(&client_aes, &header));
        assert_eq!(client_aes.get_packet_len(&header), 8);
    }

    /// Generates a client packet header with the given length
    fn generate_header_client(client_aes: &MapleAES, len: usize) -> [u8; 4] {
        let a: usize =
            (((client_aes.send_iv[3] as usize) << 8) | (client_aes.send_iv[2] as usize)) ^ 83;
        let b: usize = a ^ len;

        [
            (a % 0x100) as u8,
            (b / 0x100) as u8,
            (b % 0x100) as u8,
            (b / 0x100) as u8,
        ]
    }

    /// Checks if the given server generated header is valid
    fn is_valid_header_client(client_aes: &MapleAES, header: &[u8]) -> bool {
        if header.len() != 4 {
            return false;
        }

        (header[0] ^ client_aes.recv_iv[2]) == ((0xFFFF - client_aes.version) >> 8) as u8
            && ((header[1] ^ client_aes.recv_iv[3]) == ((0xFFFF - client_aes.version) & 0xFF) as u8)
    }
}
