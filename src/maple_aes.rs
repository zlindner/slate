use aes::Aes256;
use block_modes::block_padding::Pkcs7;
use block_modes::{BlockMode, Ecb};
use bytes::BytesMut;
use std::cmp;

type Aes256Ecb = Ecb<Aes256, Pkcs7>;

#[derive(Clone)]
pub struct MapleAES {
    pub iv: [u8; 4],
    cipher: Aes256Ecb,
    pub maple_version: u16,
}

const KEY: [u8; 32] = [
    0x13, 0x00, 0x00, 0x00, 0x08, 0x00, 0x00, 0x00, 0x06, 0x00, 0x00, 0x00, 0xB4, 0x00, 0x00, 0x00,
    0x1B, 0x00, 0x00, 0x00, 0x0F, 0x00, 0x00, 0x00, 0x33, 0x00, 0x00, 0x00, 0x52, 0x00, 0x00, 0x00,
];

const SHIFT_KEY: [u8; 256] = [
    0xEC, 0x3F, 0x77, 0xA4, 0x45, 0xD0, 0x71, 0xBF, 0xB7, 0x98, 0x20, 0xFC, 0x4B, 0xE9, 0xB3, 0xE1,
    0x5C, 0x22, 0xF7, 0x0C, 0x44, 0x1B, 0x81, 0xBD, 0x63, 0x8D, 0xD4, 0xC3, 0xF2, 0x10, 0x19, 0xE0,
    0xFB, 0xA1, 0x6E, 0x66, 0xEA, 0xAE, 0xD6, 0xCE, 0x06, 0x18, 0x4E, 0xEB, 0x78, 0x95, 0xDB, 0xBA,
    0xB6, 0x42, 0x7A, 0x2A, 0x83, 0x0B, 0x54, 0x67, 0x6D, 0xE8, 0x65, 0xE7, 0x2F, 0x07, 0xF3, 0xAA,
    0x27, 0x7B, 0x85, 0xB0, 0x26, 0xFD, 0x8B, 0xA9, 0xFA, 0xBE, 0xA8, 0xD7, 0xCB, 0xCC, 0x92, 0xDA,
    0xF9, 0x93, 0x60, 0x2D, 0xDD, 0xD2, 0xA2, 0x9B, 0x39, 0x5F, 0x82, 0x21, 0x4C, 0x69, 0xF8, 0x31,
    0x87, 0xEE, 0x8E, 0xAD, 0x8C, 0x6A, 0xBC, 0xB5, 0x6B, 0x59, 0x13, 0xF1, 0x04, 0x00, 0xF6, 0x5A,
    0x35, 0x79, 0x48, 0x8F, 0x15, 0xCD, 0x97, 0x57, 0x12, 0x3E, 0x37, 0xFF, 0x9D, 0x4F, 0x51, 0xF5,
    0xA3, 0x70, 0xBB, 0x14, 0x75, 0xC2, 0xB8, 0x72, 0xC0, 0xED, 0x7D, 0x68, 0xC9, 0x2E, 0x0D, 0x62,
    0x46, 0x17, 0x11, 0x4D, 0x6C, 0xC4, 0x7E, 0x53, 0xC1, 0x25, 0xC7, 0x9A, 0x1C, 0x88, 0x58, 0x2C,
    0x89, 0xDC, 0x02, 0x64, 0x40, 0x01, 0x5D, 0x38, 0xA5, 0xE2, 0xAF, 0x55, 0xD5, 0xEF, 0x1A, 0x7C,
    0xA7, 0x5B, 0xA6, 0x6F, 0x86, 0x9F, 0x73, 0xE6, 0x0A, 0xDE, 0x2B, 0x99, 0x4A, 0x47, 0x9C, 0xDF,
    0x09, 0x76, 0x9E, 0x30, 0x0E, 0xE4, 0xB2, 0x94, 0xA0, 0x3B, 0x34, 0x1D, 0x28, 0x0F, 0x36, 0xE3,
    0x23, 0xB4, 0x03, 0xD8, 0x90, 0xC8, 0x3C, 0xFE, 0x5E, 0x32, 0x24, 0x50, 0x1F, 0x3A, 0x43, 0x8A,
    0x96, 0x41, 0x74, 0xAC, 0x52, 0x33, 0xF0, 0xD9, 0x29, 0x80, 0xB1, 0x16, 0xD3, 0xAB, 0x91, 0xB9,
    0x84, 0x7F, 0x61, 0x1E, 0xCF, 0xC5, 0xD1, 0x56, 0x3D, 0xCA, 0xF4, 0x05, 0xC6, 0xE5, 0x08, 0x49,
];

const BLOCK_LENGTH: usize = 1460;

impl MapleAES {
    pub fn new(iv: [u8; 4], maple_version: u16) -> Self {
        let cipher = Aes256Ecb::new_from_slices(&KEY, Default::default()).unwrap();
        let maple_version: u16 = ((maple_version >> 8) & 0xff) | ((maple_version << 8) & 0xff00);

        MapleAES {
            iv,
            cipher,
            maple_version,
        }
    }

    pub fn transform(&mut self, mut data: BytesMut) -> BytesMut {
        // maplestory's 1460 byte block - 4 header bytes = 1456 bytes for body
        let mut current_block_length = BLOCK_LENGTH - 4;

        let iv_copy = [
            self.iv[0], self.iv[1], self.iv[2], self.iv[3], self.iv[0], self.iv[1], self.iv[2],
            self.iv[3], self.iv[0], self.iv[1], self.iv[2], self.iv[3], self.iv[0], self.iv[1],
            self.iv[2], self.iv[3],
        ];

        let mut i = 0;

        while i < data.len() {
            let block = cmp::min(data.len() - i, current_block_length);
            let mut xor_key = iv_copy.to_vec();

            for j in 0..block {
                if j % 16 == 0 {
                    xor_key = self.cipher.clone().encrypt_vec(&xor_key);
                }

                data[i + j] ^= xor_key[j % 16];
            }

            i += block;
            current_block_length = BLOCK_LENGTH;
        }

        // after each transform operation update the initialization vector
        self.iv = self.morph_iv();

        data
    }

    fn morph_iv(&self) -> [u8; 4] {
        let mut new_sequence: [u8; 4] = [0xf2, 0x53, 0x50, 0xc6];

        for i in 0..4 {
            let table_input = SHIFT_KEY[self.iv[i] as usize];

            // need to use wrapping add/sub as all of these operations will overflow
            new_sequence[0] = new_sequence[0]
                .wrapping_add(SHIFT_KEY[new_sequence[1] as usize].wrapping_sub(self.iv[i]));
            new_sequence[1] = new_sequence[1].wrapping_sub(new_sequence[2] ^ table_input);
            new_sequence[2] ^= SHIFT_KEY[new_sequence[3] as usize].wrapping_add(self.iv[i]);
            new_sequence[3] =
                new_sequence[3].wrapping_sub(new_sequence[0].wrapping_sub(table_input));

            let x = i32::from(new_sequence[1] & 0xff) << 8;
            let y = i32::from(new_sequence[2] & 0xff) << 16;
            let z = i32::from(new_sequence[3] & 0xff) << 24;
            // "as u32" is effectively an unsigned/zero-fill right shift by 0
            let mut val = (i32::from(new_sequence[0]) | x | y | z) as u32;
            let mut val2 = val >> 0x1d;

            // "as u32" is effectively an unsigned/zero-fill right shift by 0
            val = (val << 0x03) as u32;
            val2 |= val;

            // the below computations will always result in a value that fits in a byte
            // so downcasting as u8 should be fine here
            new_sequence[0] = (val2 & 0xff) as u8;
            new_sequence[1] = ((val2 >> 8) & 0xff) as u8;
            new_sequence[2] = ((val2 >> 16) & 0xff) as u8;
            new_sequence[3] = ((val2 >> 24) & 0xff) as u8;
        }

        new_sequence
    }

    pub fn is_valid_header(&self, header: &BytesMut) -> bool {
        ((header[0] ^ self.iv[2]) & 0xff) == ((self.maple_version >> 8) as u8 & 0xff)
            && (((header[1] ^ self.iv[3]) & 0xff) == (self.maple_version & 0xff) as u8)
    }

    pub fn create_packet_header(&self, length: u32) -> [u8; 4] {
        let mut a = u32::from(self.iv[3] & 0xff);
        a |= (u32::from(self.iv[2]) << 8) & 0xff00;
        a ^= u32::from(self.maple_version);

        let b = a ^ (((length << 8) & 0xff00) | length >> 8);

        [
            ((a >> 8) & 0xff) as u8,
            (a & 0xff) as u8,
            ((b >> 8) & 0xff) as u8,
            (b & 0xff) as u8,
        ]
    }
}
