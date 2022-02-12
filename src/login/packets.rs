use crate::{maple_aes::MapleAES, packet::Packet};

use super::handlers::LoginStatus;

pub fn handshake(ciphers: &(MapleAES, MapleAES)) -> Packet {
    let mut packet = Packet::new(18);
    packet.set_encrypt(false);
    packet.write_short(14); // packet length (0x0E)
    packet.write_short(83); // maple version (v83)
    packet.write_maple_string("1"); // maple patch version (1)
    packet.write_bytes(&ciphers.0.iv); // receive iv
    packet.write_bytes(&ciphers.1.iv); // send iv
    packet.write_byte(8); // locale
    packet
}

pub fn login_failed(reason: LoginStatus) -> Packet {
    let mut packet = Packet::new(8);
    packet.write_short(0x0);
    packet.write_int(reason as i32);
    packet.write_short(0);
    packet
}
