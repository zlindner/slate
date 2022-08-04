use oxide_core::net::{cipher::Cipher, Packet};

// handshake packet sent immediately after a client establishes a connection with the login server
// sets up client <-> server encryption via the passed initialization vectors and maple version
pub fn handshake(send: &Cipher, recv: &Cipher) -> Packet {
    let mut packet = Packet::new(0x0E);
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

pub fn login_success(id: i32, name: &String) -> Packet {
    let mut packet = Packet::new(0x00);
    packet.write_int(0);
    packet.write_short(0);
    // account id
    packet.write_int(id);
    // FIXME: gender byte => not sure if this matters, hardcoding for now
    packet.write_byte(0);
    // FIXME: gm byte (0 / 1)
    packet.write_byte(0);
    // FIXME: admin bytes (0 / 0x80)
    packet.write_byte(0);
    // country code
    packet.write_byte(0);
    packet.write_string(name);
    packet.write_byte(0);
    // is quiet banned
    packet.write_byte(0);
    // quiet ban timestamp
    packet.write_long(0);
    // creation timestamp
    packet.write_long(0);
    // remove the "select the world you want to play in"
    packet.write_int(1);
    // 0 => pin enabled, 1 => pin disabled
    //packet.write_byte(CONFIG.enable_pin);
    // 0 => register PIC, 1 => ask for PIC, 2 => disabled
    packet.write_byte(2);
    //packet.write_byte(CONFIG.enable_pic);
    packet
}

pub fn login_failed(reason: i32) -> Packet {
    let mut packet = Packet::new(0x00);
    packet.write_int(reason);
    packet.write_short(0);
    packet
}
