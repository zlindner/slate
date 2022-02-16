use crate::net::packet::Packet;
use crate::world::WorldConfig;
use crate::{crypto::maple_aes::MapleAES, world::World};

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

// TODO pass in list/vec of channels for world
pub fn world_details(world: &WorldConfig) -> Packet {
    // TODO size needs to grow based on number of channels
    let mut packet = Packet::new(46);
    packet.write_short(0x0A);
    packet.write_byte(world.id as u8);
    packet.write_maple_string(&world.name);
    packet.write_byte(world.flag as u8);
    packet.write_maple_string(&world.event_message);
    packet.write_byte(100);
    packet.write_byte(0);
    packet.write_byte(100);
    packet.write_byte(0);
    packet.write_byte(0);
    packet.write_byte(1); // number of channels TODO use size of channel vec

    // TODO iterate through channel vec, do for each channel
    packet.write_maple_string(&(world.name.to_owned() + "-1")); // TODO use channel id/index + 1
    packet.write_int(100); // TODO channel capacity
    packet.write_byte(1); // TODO world id? not sure what this is
    packet.write_byte(0); // TODO channel id
    packet.write_byte(0); // adult channel

    packet.write_short(0);

    packet
}

pub fn world_list_end() -> Packet {
    let mut packet = Packet::new(3);
    packet.write_short(0x0A);
    packet.write_byte(0xFF);
    packet
}
