pub mod maple_aes {

    use aes::Aes256;
    use block_modes::block_padding::Pkcs7;
    use block_modes::{BlockMode, Ecb};

    type Aes256Ecb = Ecb<Aes256, Pkcs7>;

    pub struct MapleAES {
        iv: [u8; 4],
        cipher: Aes256Ecb,
        maple_version: u16,
    }

    const KEY: [u8; 32] = [
        0x13, 0x00, 0x00, 0x00, 0x08, 0x00, 0x00, 0x00, 0x06, 0x00, 0x00, 0x00, 0xB4, 0x00, 0x00,
        0x00, 0x1B, 0x00, 0x00, 0x00, 0x0F, 0x00, 0x00, 0x00, 0x33, 0x00, 0x00, 0x00, 0x52, 0x00,
        0x00, 0x00,
    ];

    impl MapleAES {
        pub fn new(iv: [u8; 4], maple_version: u16) -> Self {
            let cipher = Aes256Ecb::new_from_slices(&KEY, Default::default()).unwrap();
            let maple_version: u16 =
                ((maple_version >> 8) & 0xff) | ((maple_version << 8) & 0xff00);

            MapleAES {
                iv,
                cipher,
                maple_version,
            }
        }
    }
}
