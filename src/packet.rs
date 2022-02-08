use bytes::{Buf, BufMut, BytesMut};
use std::fmt::{Display, Formatter, Result};
use std::str;

pub struct Packet {
    data: BytesMut,
}

impl Packet {
    pub fn new(size: usize) -> Self {
        Packet {
            data: BytesMut::with_capacity(size),
        }
    }

    pub fn from_bytes(bytes: BytesMut) -> Self {
        Packet { data: bytes }
    }

    pub fn get_data(&self) -> &BytesMut {
        &self.data
    }

    pub fn write_byte(&mut self, byte: u8) {
        self.data.put_u8(byte);
    }

    pub fn write_bytes(&mut self, bytes: &[u8]) {
        self.data.put_slice(bytes);
    }

    // TODO write boolean

    pub fn write_short(&mut self, num: i16) {
        self.data.put_i16_le(num);
    }

    // TODO write int
    // TODO write long
    // TODO write string

    pub fn write_maple_string(&mut self, str: &str) {
        // write the string length as an i16/short
        self.write_short(str.len() as i16);
        self.write_bytes(&str.as_bytes());
    }

    pub fn read_bytes(&mut self, num_bytes: usize) -> BytesMut {
        self.data.split_to(num_bytes)
    }

    pub fn read_short(&mut self) -> i16 {
        self.data.get_i16_le()
    }

    pub fn read_maple_string(&mut self) -> String {
        let length = self.read_short() as usize;
        let bytes = self.data.split_to(length);

        str::from_utf8(&bytes).unwrap().to_string()
    }

    pub fn advance(&mut self, num_bytes: usize) {
        self.data.advance(num_bytes);
    }
}

// BytesMut refuses to format properly without this...
impl Display for Packet {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "[")?;

        let len = self.data.len();

        for i in 0..len {
            write!(f, "{:02X}", self.data[i])?;

            if i != len - 1 {
                write!(f, ", ")?;
            }
        }

        write!(f, "]")?;
        Ok(())
    }
}
