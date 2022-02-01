pub mod packet {

    #[derive(Debug)]
    pub struct Packet {
        data: Vec<u8>,
    }

    impl Packet {
        pub fn new(bytes: usize) -> Self {
            Packet {
                data: Vec::with_capacity(bytes),
            }
        }

        pub fn bytes(self) -> Vec<u8> {
            self.data
        }

        pub fn write_byte(&mut self, byte: u8) {
            self.data.push(byte);
        }

        pub fn write_bytes(&mut self, bytes: &[u8]) {
            self.data.extend_from_slice(bytes);
        }

        // TODO write boolean

        // writes a short (signed 16 bit integer) to the packet
        pub fn write_short(&mut self, num: i16) {
            let bytes = num.to_le_bytes();
            self.data.extend_from_slice(&bytes);
        }

        // TODO write int
        // TODO write long
        // TODO write string

        pub fn write_maple_string(&mut self, str: &str) {
            // write the string length as an i16/short
            self.write_short(str.len() as i16);

            let bytes = str.as_bytes();
            self.data.extend_from_slice(&bytes);
        }
    }
}
