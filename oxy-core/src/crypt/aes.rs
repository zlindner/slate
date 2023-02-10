use crate::net::Packet;
use aes::{
    cipher::{generic_array::GenericArray, BlockEncrypt, KeyInit},
    Aes256,
};
use rand::random;

pub struct MapleAES {
    cipher: Aes256,
    send_iv: [u8; 4],
    recv_iv: [u8; 4],
    version: u16,
}

impl MapleAES {
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

    pub fn new_with_iv(version: u16, send_iv: [u8; 4], recv_iv: [u8; 4]) -> Self {
        let mut key = super::KEY;
        let key = GenericArray::from_mut_slice(&mut key);

        Self {
            cipher: Aes256::new(key),
            send_iv,
            recv_iv,
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

    /// Transforms data with the passed iv, and returns a new iv
    fn transform(&self, data: &mut [u8], iv: [u8; 4]) -> [u8; 4] {
        let mut remaining = data.len();
        let mut start = 0;
        let mut block_length = 0x5B0;

        while remaining > 0 {
            let mut iv = iv.repeat(4);
            let mut iv = GenericArray::from_mut_slice(&mut iv);

            if remaining < block_length {
                block_length = remaining;
            }

            for i in start..(start + block_length) {
                if (i - start) % iv.len() == 0 {
                    self.cipher.encrypt_block(&mut iv);
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

        for i in 0..4 {
            let byte = iv[i];
            new_iv[0] =
                new_iv[0].wrapping_add(shift_bytes[(new_iv[1] & 0xFF) as usize].wrapping_sub(byte));
            new_iv[1] =
                new_iv[1].wrapping_sub(new_iv[2] ^ shift_bytes[(byte & 0xFF) as usize] & 0xFF);
            new_iv[2] = new_iv[2] ^ (shift_bytes[(new_iv[3] & 0xFF) as usize].wrapping_add(byte));
            new_iv[3] = new_iv[3].wrapping_add(
                (shift_bytes[(byte & 0xFF) as usize] & 0xFF).wrapping_sub(new_iv[0] & 0xFF),
            );

            let mut mask = 0usize;
            mask |= (new_iv[0] as usize) & 0xFF;
            mask |= ((new_iv[1] as usize) << 8) & 0xFF00;
            mask |= ((new_iv[2] as usize) << 16) & 0xFF0000;
            mask |= ((new_iv[3] as usize) << 24) & 0xFF000000;
            mask = (mask >> 0x1D) | (mask << 3);

            for j in 0..4 {
                new_iv[j] = ((mask >> (8 * j)) & 0xFF) as u8;
            }
        }

        new_iv
    }

    /// Creates the handshake packet for sharing IVs with the client
    pub fn get_handshake(&self) -> Packet {
        let mut handshake = Packet::new();
        handshake.write_short(0x0E);
        handshake.write_short(83); // maple version
        handshake.write_string("1"); // maple patch version
        handshake.write_bytes(&self.recv_iv);
        handshake.write_bytes(&self.send_iv);
        handshake.write_byte(8); // locale
        handshake
    }

    /// Checks if the given header is valid
    pub fn is_valid_header(&self, header: &[u8]) -> bool {
        ((header[0] ^ self.recv_iv[2]) & 0xFF) == ((self.version >> 8) as u8 & 0xFF)
            && (((header[1] ^ self.recv_iv[3]) & 0xFF) == (self.version & 0xFF) as u8)
    }

    /// Generates a packet header with the given length
    pub fn build_header(&self, len: usize) -> [u8; 4] {
        let mut iiv: u32 = self.send_iv[3] as u32 & 0xFF;
        iiv |= ((self.send_iv[2] as u32) << 8) & 0xFF00;
        iiv ^= 0xFFFF - self.version as u32;
        let mlength = (((len as u32) << 8) & 0xFF00) | ((len as u32) >> 8);
        let xored_iv = iiv ^ mlength;

        [
            (iiv >> 8) as u8 & 0xFF,
            iiv as u8 & 0xFF,
            (xored_iv >> 8) as u8 & 0xFF,
            xored_iv as u8 & 0xFF,
        ]
    }

    /// Gets the packet length from the given header
    pub fn get_packet_len(&self, header: &[u8]) -> i16 {
        (header[0] as i16 + ((header[1] as i16) << 8))
            ^ (header[2] as i16 + ((header[3] as i16) << 8))
    }
}
