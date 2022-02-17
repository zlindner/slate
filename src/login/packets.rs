use bytes::Buf;

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

pub fn world_details(world: &World) -> Packet {
    let config = &world.config;

    // calculate the number of bytes for the packet
    let mut len = 16 + config.name.len() + config.event_message.len();
    // add 2 bytes for channel id (in case # of channels > 10)
    len += (9 + config.name.len() + 2) * world.channels.len();

    let mut packet = Packet::new(len);
    packet.write_short(0x0A);
    packet.write_byte(config.id as u8);
    packet.write_maple_string(&config.name);
    packet.write_byte(config.flag as u8);
    packet.write_maple_string(&config.event_message);
    packet.write_byte(100);
    packet.write_byte(0);
    packet.write_byte(100);
    packet.write_byte(0);
    packet.write_byte(0);
    packet.write_byte(world.channels.len() as u8);

    for channel in world.channels.iter() {
        packet.write_maple_string(&(config.name.to_owned() + &(channel.id + 1).to_string()));
        packet.write_int(100); // TODO channel capacity, not sure if this is max allowed or currently connected?
        packet.write_byte(1); // TODO world id? not sure what this is
        packet.write_byte(channel.id as u8);
        packet.write_byte(0); // adult channel
    }

    packet.write_short(0);
    packet
}

pub fn world_list_end() -> Packet {
    let mut packet = Packet::new(3);
    packet.write_short(0x0A);
    packet.write_byte(0xFF);
    packet
}
