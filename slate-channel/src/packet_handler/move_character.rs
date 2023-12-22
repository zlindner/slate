use crate::session::ChannelSession;
use slate_data::maple;
use slate_net::Packet;

/// Channel server: move character packet (0x29)
/// Called when a character is moved
pub async fn handle(mut packet: Packet, session: &mut ChannelSession) -> anyhow::Result<()> {
    packet.skip(9);
    let packet_copy = packet.clone();
    let num_commands = packet.read_byte();

    let mut new_pos: Option<(i32, i32)> = None;
    let mut new_stance: Option<u8> = None;

    for _ in 0..num_commands {
        let command = packet.read_byte();

        match command {
            // Absolute movement -- only important for the server
            0 | 5 | 17 => {
                let x = packet.read_short();
                let y = packet.read_short();
                new_pos = Some((x.into(), y.into()));
                packet.skip(6);
                let stance = packet.read_byte();
                new_stance = Some(stance);
                packet.skip(2);
            }
            // Relative movement -- server only cares about stance
            1 | 2 | 6 | 12 | 13 | 16 | 18 | 19 | 20 | 22 => {
                packet.skip(4);
                let stance = packet.read_byte();
                new_stance = Some(stance);
                packet.skip(2);
            }
            // Teleport movement -- server only cares about stance
            3 | 4 | 7 | 8 | 9 | 11 => {
                packet.skip(8);
                let stance = packet.read_byte();
                new_stance = Some(stance);
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
                new_stance = Some(stance);
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

    // Character hasn't moved -- do nothing
    if new_pos.is_none() && new_stance.is_none() {
        return Ok(());
    }

    // Get the map as read-only
    {
        let map = session.state.get_map(session.map_id.unwrap());
        let character = map.characters.get(&session.character_id.unwrap()).unwrap();

        let broadcast = maple::map::Broadcast {
            packet: move_player(session.character_id.unwrap(), packet_copy),
            sender_id: character.data.id,
            send_to_sender: false,
        };
        map.broadcast_tx.send(broadcast)?;
    }

    // Get the map as read-write
    {
        let mut map = session.state.get_map_mut(session.map_id.unwrap());
        let character = map
            .characters
            .get_mut(&session.character_id.unwrap())
            .unwrap();

        character.pos = new_pos.unwrap_or(character.pos);
        character.stance = new_stance.unwrap_or(character.stance);
    }

    Ok(())
}

///
fn move_player(character_id: i32, movement_data: Packet) -> Packet {
    let mut packet = Packet::new(0xB9);
    packet.write_int(character_id);
    packet.write_int(0);
    packet.write_bytes(&movement_data.bytes);
    packet
}
