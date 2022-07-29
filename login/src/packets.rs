use oxide_core::net::{cipher::Cipher, Packet};

// handshake packet sent immediately after a client establishes a connection with the login server
// sets up client <-> server encryption via the passed initialization vectors and maple version
pub fn handshake(send: &Cipher, recv: &Cipher) -> Packet {
    let mut packet = Packet::new();
    // packet length (0x0E)
    packet.write_short(0x0E);
    // maple version
    packet.write_short(83);
    // maple patch version
    packet.write_string("1");
    // initialization vector for receive cipher
    packet.write_bytes(&recv.iv);
    // initialization vector for send cipher
    packet.write_bytes(&send.iv);
    // locale
    packet.write_byte(8);
    packet
}
