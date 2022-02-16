use crate::crypto::maple_aes::MapleAES;
use crate::packet::Packet;

use super::handlers::{Account, LoginError};

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

pub fn login_failed(reason: LoginError) -> Packet {
    let mut packet = Packet::new(8);
    packet.write_short(0x0);
    packet.write_int(reason as i32);
    packet.write_short(0);
    packet
}

pub fn login_success(account: &Account) -> Packet {
    let mut packet = Packet::new(42 + account.name.len());
    packet.write_short(0x00);
    packet.write_int(0);
    packet.write_short(0);
    packet.write_int(account.id);
    packet.write_byte(0); // FIXME: gender byte => not sure if this matters, hardcoding for now
    packet.write_byte(0); // FIXME: gm byte (0 / 1)
    packet.write_byte(0); // FIXME: admin bytes (0 / 0x80)
    packet.write_byte(0); // country code
    packet.write_maple_string(&account.name);
    packet.write_byte(0);
    packet.write_byte(0); // is quiet ban
    packet.write_long(0); // is quiet ban timestamp
    packet.write_long(0); // creation timestamp
    packet.write_int(1); // remove the world selector
    packet.write_byte(1); // FIXME: 0 => pin enabled, 1 => pin disabled
    packet.write_byte(2); // FIXME: 0 => register PIC, 1 => ask for PIC, 2 => disabled
    packet
}
