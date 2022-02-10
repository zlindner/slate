use crate::{maple_aes::MapleAES, packet::Packet};

pub fn handshake(ciphers: &(MapleAES, MapleAES)) -> Packet {
    let mut packet = Packet::new(18);
    packet.write_short(14); // packet length (0x0E)
    packet.write_short(83); // maple version (v83)
    packet.write_maple_string("1"); // maple patch version (1)
    packet.write_bytes(&ciphers.0.iv); // receive iv
    packet.write_bytes(&ciphers.1.iv); // send iv
    packet.write_byte(8); // locale
    packet
}
