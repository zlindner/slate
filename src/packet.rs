use bytes::{BufMut, BytesMut};

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
}
