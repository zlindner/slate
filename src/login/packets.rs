use crate::character::Character;
use crate::net::packet::Packet;
use crate::world::CapacityStatus;
use crate::{crypto::cipher::Cipher, world::World};

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

// login success packet
pub fn login_success(id: i32, name: &String) -> Packet {
    let mut packet = Packet::new();
    packet.write_short(0x00);
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
    packet.write_int(1);
    // FIXME: 0 => pin enabled, 1 => pin disabled
    packet.write_byte(1);
    // FIXME: 0 => register PIC, 1 => ask for PIC, 2 => disabled
    packet.write_byte(2);
    packet
}

// login failed packet containing the reason why the login failed
pub fn login_failed(reason: i32) -> Packet {
    let mut packet = Packet::new();
    packet.write_short(0x00);
    // reason code
    packet.write_int(reason);
    packet.write_short(0);
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

pub fn world_status(status: CapacityStatus) -> Packet {
    let mut packet = Packet::new(4);
    packet.write_short(0x03);
    packet.write_short(status as i16);
    packet
}

pub fn character_list() -> Packet {
    let mut packet = Packet::new(9);
    packet.write_short(0x0B);
    packet.write_byte(0); // status

    // TODO need to add data for each character

    packet.write_byte(0); // number of characters
    packet.write_byte(2); // FIXME: 0 => register PIC, 1 => ask for PIC, 2 => disabled
    packet.write_int(3); // number of character slots allowed for this client

    packet
}

pub fn character_name(name: &str, valid: bool) -> Packet {
    let mut packet = Packet::new(name.len() + 5);
    packet.write_short(0x0D);
    packet.write_maple_string(name);
    packet.write_byte(!valid as u8); // name is taken => !valid
    packet
}

pub fn create_character(character: &Character) -> Packet {
    let mut packet = Packet::new(256);
    packet.write_short(0x0E);
    packet.write_byte(0);

    add_character_stats(&mut packet, character);
    add_character_style(&mut packet, character);
    add_character_equipment(&mut packet, character);

    packet.write_byte(0); // view all

    // TODO if gm or gm job, write_byte(0) and return;

    packet.write_byte(1); // world rank enabled
    packet.write_int(character.rank.rank);
    packet.write_int(character.rank.rank_move); // positive => upwards, negative => downwards
    packet.write_int(character.rank.job_rank);
    packet.write_int(character.rank.job_rank_move); // positive => upwards, negative => downwards

    log::debug!("create_character packet size: {}", packet.data.len());
    packet
}

fn add_character_stats(packet: &mut Packet, character: &Character) {
    packet.write_int(character.id);

    let mut padded_name = String::from(character.name.clone());

    for _ in padded_name.len()..13 {
        padded_name.push('\0');
    }

    packet.write_fixed_string(&padded_name);

    packet.write_byte(character.style.gender as u8);
    packet.write_byte(character.style.skin_colour as u8);
    packet.write_int(character.style.face);
    packet.write_int(character.style.hair);

    // pets
    for _ in 0..3 {
        // TODO get character.pet(i)
        // if not null -> write_long(pet.id)
        packet.write_long(0);
    }

    // stats
    packet.write_byte(character.stats.level as u8);
    packet.write_short(character.job as i16);
    packet.write_short(character.stats.str as i16);
    packet.write_short(character.stats.dex as i16);
    packet.write_short(character.stats.int as i16);
    packet.write_short(character.stats.luk as i16);
    packet.write_short(character.stats.hp as i16);
    packet.write_short(character.stats.max_hp as i16);
    packet.write_short(character.stats.mp as i16);
    packet.write_short(character.stats.max_mp as i16);
    packet.write_short(character.stats.ap as i16);
    // TODO can add remaining skill info here for evan
    packet.write_short(0); // TODO remainingSp
    packet.write_int(character.stats.exp);
    packet.write_short(character.stats.fame as i16);
    packet.write_int(character.stats.gacha_exp);
    packet.write_int(character.map);
    packet.write_byte(character.spawn_point as u8);
    packet.write_int(0);
}

fn add_character_style(packet: &mut Packet, character: &Character) {
    packet.write_byte(character.style.gender as u8);
    packet.write_byte(character.style.skin_colour as u8);
    packet.write_int(character.style.face);
    packet.write_byte(1); // mega?
    packet.write_int(character.style.hair);
}

fn add_character_equipment(packet: &mut Packet, _: &Character) {
    packet.write_byte(0x05); // 5
    packet.write_int(1040010);

    packet.write_byte(0x06); // 6
    packet.write_int(1060006);

    packet.write_byte(0x07); // 7
    packet.write_int(1072038);

    packet.write_byte(0x0B); // 11
    packet.write_int(1322005);

    packet.write_byte(0xFF);

    // masked equips?
    packet.write_byte(0xFF);

    packet.write_int(0); // FIXME if item w/ id/pos? -111 is not null add its id here?

    // pet stuff?
    for _ in 0..3 {
        // TODO if pet.get(i) != null write_int(pet(i) item id)
        packet.write_int(0);
    }
}
