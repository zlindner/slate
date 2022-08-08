use oxide_core::{
    net::{cipher::Cipher, Packet},
    Character,
};

pub enum PinOperation {
    Accepted,
    Register,
    RequestAfterFailure,
    ConnectionFailed,
    Request,
}

pub enum PicOperation {
    Success = 0x00,
    UnknownError = 0x09,
    InvalidPic = 0x14,
    GuildMasterError = 0x16,
    PendingWorldTransferError = 0x1A,
    HasFamilyError = 0x1D,
}

// handshake packet sent immediately after a client establishes a connection with the login server
// sets up client <-> server encryption via the passed initialization vectors and maple version
pub fn handshake(send: &Cipher, recv: &Cipher) -> Packet {
    let mut packet = Packet::new();
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
    // remove the "select the world you want to play in"
    packet.write_int(1);
    // 0 => pin enabled, 1 => pin disabled
    packet.write_byte(1);
    //packet.write_byte(CONFIG.enable_pin);
    // 0 => register PIC, 1 => ask for PIC, 2 => disabled
    packet.write_byte(2);
    //packet.write_byte(CONFIG.enable_pic);
    packet
}

pub fn login_failed(reason: i32) -> Packet {
    let mut packet = Packet::new();
    packet.write_short(0x00);
    packet.write_int(reason);
    packet.write_short(0);
    packet
}

// packet for various PIN operations
// 0 => PIN was accepted
// 1 => register a new PIN
// 2 => invalid PIN / re-enter
// 3 => connection failed due to system error
// 4 => enter pin
pub fn pin_operation(op: PinOperation) -> Packet {
    let mut packet = Packet::new();
    packet.write_short(0x06);
    packet.write_byte(op as u8);
    packet
}

pub fn pin_registered() -> Packet {
    let mut packet = Packet::new();
    packet.write_short(0x07);
    packet.write_byte(0);
    packet
}

/*
// contains info for the given world displayed to the client in the world/server list
pub fn world_details(world: &World) -> Packet {
    let mut packet = Packet::new();
    packet.write_short(0x0A);
    packet.write_byte(world.config.id as u8);
    packet.write_string(&world.config.name);
    packet.write_byte(world.config.flag as u8);
    packet.write_string(&world.config.event_message);
    packet.write_byte(100);
    packet.write_byte(0);
    packet.write_byte(100);
    packet.write_byte(0);
    packet.write_byte(0);
    packet.write_byte(world.channels.len() as u8);

    for channel in world.channels.iter() {
        let channel_name = &(world.config.name.to_owned() + &(channel.id + 1).to_string());
        packet.write_string(channel_name);
        // TODO channel capacity, not sure if this is max allowed or currently connected?
        packet.write_int(100);
        // TODO world id? not sure what this is
        packet.write_byte(1);
        packet.write_byte(channel.id as u8);
        // is adult channel
        packet.write_byte(0);
    }

    packet.write_short(0);
    packet
}
 */

// FIXME currently hardcoded
// contains info for the given world displayed to the client in the world/server list
pub fn world_details_temp() -> Packet {
    let mut packet = Packet::new();
    packet.write_short(0x0A);
    packet.write_byte(0);
    packet.write_string("Scania");
    packet.write_byte(0);
    packet.write_string("Scania!");
    packet.write_byte(100);
    packet.write_byte(0);
    packet.write_byte(100);
    packet.write_byte(0);
    packet.write_byte(0);

    let channels = 8;
    packet.write_byte(channels);

    for channel in 1..=channels {
        let channel_name = format!("Scania{}", channel);
        packet.write_string(&channel_name);
        // TODO channel capacity, not sure if this is max allowed or currently connected?
        packet.write_int(100);
        // TODO world id? not sure what this is
        packet.write_byte(1);
        packet.write_byte(channel);
        // is adult channel
        packet.write_byte(0);
    }

    packet.write_short(0);
    packet
}

// packet indicating that we have sent details for all available worlds
pub fn world_list_end() -> Packet {
    let mut packet = Packet::new();
    packet.write_short(0x0A);
    packet.write_byte(0xFF);
    packet
}

// pre-selects a world for the client after loading the world/server list
// TODO according to GMS, this should be the "most active" world
pub fn world_select(world_id: i32) -> Packet {
    let mut packet = Packet::new();
    packet.write_short(0x1A);
    packet.write_int(world_id);
    packet
}

// FIXME currently hardcoded
pub fn view_recommended_temp() -> Packet {
    let mut packet = Packet::new();
    packet.write_short(0x1B);
    packet.write_byte(1);

    packet.write_int(0);
    packet.write_string("Welcome to Scania!");

    packet
}

// FIXME currently hardcoded
pub fn world_status_temp() -> Packet {
    let mut packet = Packet::new();
    packet.write_short(0x03);
    packet.write_short(0);
    packet
}

/*
// displays the "Recommended World" text for each world when the client selects "View Recommended"
pub fn view_recommended(worlds: &Vec<World>) -> Packet {
    let mut packet = Packet::new();
    packet.write_short(0x1B);
    packet.write_byte(worlds.len() as u8);

    for world in worlds.iter() {
        packet.write_int(world.config.id);
        packet.write_string(&world.config.recommended);
    }

    packet
}*/

pub fn character_list(characters: &Vec<Character>) -> Packet {
    let mut packet = Packet::new();
    packet.write_short(0x0B);
    // status code
    packet.write_byte(0);
    // number of characters
    packet.write_byte(characters.len() as u8);

    for character in characters.iter() {
        write_character(character, &mut packet, false);
    }

    // 0 => register PIC, 1 => ask for PIC, 2 => disabled
    packet.write_byte(2);
    // packet.write_byte(CONFIG.enable_pic); FIXME
    // number of character slots
    // TODO should be configurable via config/oxide.toml
    packet.write_int(3);
    packet
}

fn write_character(character: &Character, packet: &mut Packet, view_all: bool) {
    write_character_stats(character, packet);
    write_character_style(character, packet);
    write_character_equipment(character, packet);

    if !view_all {
        packet.write_byte(0);
    }

    // TODO check for gm job as well?
    if character.gm > 1 {
        packet.write_byte(0);
        return;
    }

    // TODO load from oxide.toml?
    let enable_rankings = true;
    packet.write_byte(enable_rankings as u8);

    if enable_rankings {
        packet.write_int(character.rank);
        // positive/negative indicate direction of move
        packet.write_int(character.rank_move);
        packet.write_int(character.job_rank);
        // positive/negative indicate direction of move
        packet.write_int(character.job_rank_move);
    }
}

fn write_character_stats(character: &Character, packet: &mut Packet) {
    packet.write_int(character.id);

    let mut padded_name = String::from(character.name.clone());

    for _ in padded_name.len()..13 {
        padded_name.push('\0');
    }

    packet.write_fixed_string(&padded_name);

    // style
    packet.write_byte(character.gender as u8);
    packet.write_byte(character.skin_colour as u8);
    packet.write_int(character.face);
    packet.write_int(character.hair);

    // pets
    for i in 0..3 {
        match character.pets.get(i) {
            Some(pet) => packet.write_long(pet.id.into()),
            None => packet.write_long(0),
        }
    }

    // stats
    packet.write_byte(character.level as u8);
    packet.write_short(character.job);
    packet.write_short(character.str);
    packet.write_short(character.dex);
    packet.write_short(character.int);
    packet.write_short(character.luk);
    packet.write_short(character.hp);
    packet.write_short(character.max_hp);
    packet.write_short(character.mp);
    packet.write_short(character.max_mp);
    packet.write_short(character.ap);
    // TODO can add remaining skill info here for evan
    // TODO get characters remaining sp for job
    packet.write_short(0);
    packet.write_int(character.exp);
    packet.write_short(character.fame as i16);
    packet.write_int(character.gacha_exp);
    packet.write_int(character.map);
    packet.write_byte(character.spawn_point as u8);
    packet.write_int(0);
}

fn write_character_style(character: &Character, packet: &mut Packet) {
    packet.write_byte(character.gender as u8);
    packet.write_byte(character.skin_colour as u8);
    packet.write_int(character.face);
    // TODO add mega parameter => I think for diplaying char in megaphone message?
    packet.write_byte(1);
    packet.write_int(character.hair);
}

fn write_character_equipment(character: &Character, packet: &mut Packet) {
    packet.write_byte(0x05); // 5
    packet.write_int(1040010);

    packet.write_byte(0x06); // 6
    packet.write_int(1060006);

    packet.write_byte(0x07); // 7
    packet.write_int(1072038);

    packet.write_byte(0x0B); // 11
    packet.write_int(1322005);

    packet.write_byte(0xFF);

    // TODO masked equips
    packet.write_byte(0xFF);

    // FIXME
    // Item cWeapon = equip.getItem((short) -111);
    // p.writeInt(cWeapon != null ? cWeapon.getItemId() : 0);
    packet.write_int(0);

    // pets
    for i in 0..3 {
        match character.pets.get(i) {
            Some(pet) => packet.write_int(pet.item_id),
            None => packet.write_int(0),
        }
    }
}

pub fn new_character(character: &Character) -> Packet {
    let mut packet = Packet::new();
    packet.write_short(0x0E);
    packet.write_byte(0);
    write_character(character, &mut packet, false);
    packet
}

pub fn character_name(name: &String, valid: bool) -> Packet {
    let mut packet = Packet::new();
    packet.write_short(0x0D);
    packet.write_string(name);
    packet.write_byte(!valid as u8);
    packet
}

pub fn delete_character(character_id: i32, op: PicOperation) -> Packet {
    let mut packet = Packet::new();
    packet.write_short(0x0F);
    packet.write_int(character_id);
    packet.write_byte(op as u8);
    packet
}

pub fn channel_server_ip(character_id: i32) -> Packet {
    let mut packet = Packet::new();
    packet.write_short(0x0C);
    packet.write_short(0);
    // FIXME correct ip for selected world (without port)
    packet.write_bytes(&[0xC0, 0xA8, 0x0, 0x25]);
    // FIXME correct port for selected channel
    packet.write_short(10000);
    packet.write_int(character_id);
    packet.write_bytes(&[0, 0, 0, 0, 0]);
    packet
}
