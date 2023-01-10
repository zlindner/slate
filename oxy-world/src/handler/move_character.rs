use crate::{client::WorldClient, Shared};
use anyhow::Result;
use oxy_core::net::Packet;

/// World server: move character packet (0x29)
/// Called when the client moves their character
pub async fn handle(mut packet: Packet, client: &mut WorldClient, shared: &Shared) -> Result<()> {
    packet.skip(9);
    let packet_copy = packet.clone();
    let num_commands = packet.read_byte();

    let map = shared.get_map(client.map_id);

    // TODO would .get() with a clone, then insert at the end be faster?
    let mut character = map.characters.get_mut(&client.character_id).unwrap();

    for _ in 0..num_commands {
        let command = packet.read_byte();

        match command {
            // Absolute movement -- only important for the server
            0 | 5 | 17 => {
                let x = packet.read_short();
                let y = packet.read_short();
                character.position = (x.into(), y.into());
                packet.skip(6);
                let stance = packet.read_byte();
                character.stance = stance.into();
                packet.skip(2);
            }
            // Relative movement -- server only cares about stance
            1 | 2 | 6 | 12 | 13 | 16 | 18 | 19 | 20 | 22 => {
                packet.skip(4);
                let stance = packet.read_byte();
                character.stance = stance.into();
                packet.skip(2);
            }
            // Teleport movement -- server only cares about stance
            3 | 4 | 7 | 8 | 9 | 11 => {
                packet.skip(8);
                let stance = packet.read_byte();
                character.stance = stance.into();
            }
            14 => {
                packet.skip(9);
            }
            10 => {
                packet.skip(1);
            }
            // Jump-down -- server only cares about stance
            15 => {
                packet.skip(12);
                let stance = packet.read_byte();
                character.stance = stance.into();
                packet.skip(2);
            }
            21 => {
                packet.skip(3);
            }
            _ => {
                log::debug!("Unhandled movement command: {}", command);
            }
        }
    }

    // TODO we should build a vec in the above loop and only broadcast movement packets that matter
    // on the client (ex. don't need to send absolute movement)
    let response = move_player(client.session.character_id, packet_copy);
    client.broadcast(response, false).await?;

    Ok(())
}

///
fn move_player(character_id: i32, movement_data: Packet) -> Packet {
    let mut packet = Packet::new();
    packet.write_short(0xB9);
    packet.write_int(character_id);
    packet.write_int(0);
    packet.write_bytes(&movement_data.bytes);
    packet
}
