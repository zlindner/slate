use crate::session::ChannelSession;
use slime_data::nx;
use slime_net::Packet;

/// Channel server: quest action packet (0x6B)
/// Called when a quest action is performed (accept, ...)
pub async fn handle(mut packet: Packet, session: &mut ChannelSession) -> anyhow::Result<()> {
    let action = packet.read_byte();
    let quest_id = packet.read_short();

    let quest = nx::Quest::load(quest_id as i32)?;
    let map = session.state.get_map(session.map_id.unwrap());
    let character = map.characters.get(&session.character_id.unwrap()).unwrap();

    match action {
        // Restore lost item
        0 => {}
        // Start quest
        1 => {
            let npc_id = packet.read_int();

            // TODO check if npc is nearby

            if quest.start(character, npc_id) {
                // start the quest

                session
                    .stream
                    .write_packet(update_quest(quest_id, false))
                    .await?;

                // TODO if quest has info number requirement (and info number > 0), also send update_quest with info_update true

                session
                    .stream
                    .write_packet(update_quest_info(quest_id, npc_id))
                    .await?;
            }
        }
        // Complete quest
        2 => {
            let npc_id = packet.read_int();

            // TODO check if npc is nearby

            // TODO verify quest is started (in-memory?)
        }
        // Forfeit quest
        3 => {}
        // Start scripted quest
        4 => {}
        // Complete scripted quest
        5 => {}
        _ => {
            log::error!("Invalid quest action: {}", action);
        }
    }

    Ok(())
}

// Update quest packet
fn update_quest(quest_id: i16, info_update: bool) -> Packet {
    let mut packet = Packet::new(0x27);
    packet.write_byte(1);
    packet.write_short(quest_id);

    if info_update {
        // packet.write_byte(1);
    } else {
        // TODO not started (0), started (1), completed (2);
        packet.write_byte(1);
    }

    // TODO string progress data
    packet.write_string("");
    packet.write_bytes(&[0, 0, 0, 0, 0]);
    packet
}

fn update_quest_info(quest_id: i16, npc_id: i32) -> Packet {
    let mut packet = Packet::new(0xD3);
    packet.write_byte(8); // 0x0A in v95
    packet.write_short(quest_id);
    packet.write_int(npc_id);
    packet.write_int(0);
    packet
}

/*
public static Packet updateQuest(Character chr, QuestStatus qs, boolean infoUpdate) {
        final OutPacket p = OutPacket.create(SendOpcode.SHOW_STATUS_INFO);
        p.writeByte(1);
        if (infoUpdate) {
            QuestStatus iqs = chr.getQuest(qs.getInfoNumber());
            p.writeShort(iqs.getQuestID());
            p.writeByte(1);
            p.writeString(iqs.getProgressData());
        } else {
            p.writeShort(qs.getQuest().getId());
            p.writeByte(qs.getStatus().getId());
            p.writeString(qs.getProgressData());
        }
        p.skip(5);
        return p;
    }

*/
