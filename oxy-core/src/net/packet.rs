use bytes::{Buf, BufMut, Bytes, BytesMut};
use std::fmt::{Display, Formatter};

pub struct Packet {
    pub bytes: BytesMut,
}

impl Packet {
    pub fn new() -> Self {
        Self {
            bytes: BytesMut::with_capacity(1024),
        }
    }

    pub fn wrap(bytes: BytesMut) -> Self {
        Self { bytes }
    }

    pub fn write_byte(&mut self, byte: u8) {
        self.bytes.put_u8(byte);
    }

    pub fn write_bytes(&mut self, bytes: &[u8]) {
        self.bytes.put_slice(bytes);
    }

    pub fn write_short(&mut self, short: i16) {
        self.bytes.put_i16_le(short);
    }

    pub fn write_int(&mut self, int: i32) {
        self.bytes.put_i32_le(int);
    }

    pub fn write_long(&mut self, long: i64) {
        self.bytes.put_i64_le(long);
    }

    pub fn write_string(&mut self, string: &str) {
        self.write_short(string.len() as i16);
        self.write_bytes(string.as_bytes());
    }

    pub fn write_fixed_string(&mut self, string: &str) {
        self.write_bytes(string.as_bytes());
    }

    pub fn read_byte(&mut self) -> u8 {
        self.bytes.get_u8()
    }

    pub fn read_bytes(&mut self, len: usize) -> Bytes {
        self.bytes.split_to(len).freeze()
    }

    pub fn read_short(&mut self) -> i16 {
        self.bytes.get_i16_le()
    }

    pub fn read_int(&mut self) -> i32 {
        self.bytes.get_i32_le()
    }

    pub fn read_string(&mut self) -> String {
        let len = self.read_short() as usize;
        let bytes = self.bytes.split_to(len);
        std::str::from_utf8(&bytes).unwrap().into()
    }

    pub fn skip(&mut self, num: usize) {
        self.bytes.advance(num)
    }

    pub fn len(&self) -> usize {
        self.bytes.len()
    }

    pub fn remaining(&self) -> usize {
        self.bytes.remaining()
    }
}

impl Display for Packet {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "[")?;

        let len = self.bytes.len();

        for i in 0..len {
            write!(f, "{:02X}", self.bytes[i])?;

            if i != len - 1 {
                write!(f, ", ")?;
            }
        }

        write!(f, "]")?;
        Ok(())
    }
}
