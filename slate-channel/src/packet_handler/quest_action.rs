use crate::session::ChannelSession;
use slate_data::{
    maple, nx,
    packet::{self, SpecialEffect},
};
use slate_net::Packet;
use sqlx::types::chrono::{Local, Utc};

/// Channel server: quest action packet (0x6B)
/// Called when a quest action is performed (start, complete, forfeit, etc.)
pub async fn handle(mut packet: Packet, session: &mut ChannelSession) -> anyhow::Result<()> {
    let action = packet.read_byte();
    let quest_id = packet.read_short();

    let quest = nx::Quest::load(quest_id)?;
    let map = session.state.get_map(session.map_id.unwrap());
    let character = map.characters.get(&session.character_id.unwrap()).unwrap();

    match action {
        // Restore lost item
        0 => {}
        // Start quest
        1 => {
            let npc_id = packet.read_int();

            if quest.start(character, npc_id) {
                // We no longer need the map, drop it to release the lock
                drop(map);

                session
                    .stream
                    .write_packet(update_quest(quest_id, false))
                    .await?;

                // TODO if quest has info number requirement (and info number > 0), also send update_quest with info_update true

                // TODO this doesn't seem to do anything...
                /*session
                .stream
                .write_packet(update_quest_info(quest_id, npc_id))
                .await?;*/

                // TODO update quest status in db
            }
        }
        // Complete quest
        2 => {
            let npc_id = packet.read_int();
            let mut selection: Option<i16> = None;

            if packet.remaining() >= 2 {
                selection = Some(packet.read_short());
            }

            if quest.complete(character, npc_id, selection) {
                // Show the quest completed effect to all other players in the map
                let broadcast = maple::map::Broadcast {
                    packet: packet::show_foreign_effect(
                        character.data.id,
                        SpecialEffect::QuestComplete,
                    ),
                    sender_id: character.data.id,
                    send_to_sender: false, // TODO could this be true and not have to send show_special_effect?
                };
                map.broadcast_tx.send(broadcast)?;

                // We no longer need the map, drop it to release the lock
                drop(map);

                // Show the quest completed effect to the character
                session
                    .stream
                    .write_packet(packet::show_special_effect(SpecialEffect::QuestComplete))
                    .await?;

                // TODO if quest is not repetable and not exploitable, reward character with quest fame (and fame?)

                session
                    .stream
                    .write_packet(complete_quest(quest_id))
                    .await?;

                // TODO only do below if quest doesn't have next action

                // TODO this doesn't seem to do anything...
                /*session
                .stream
                .write_packet(update_quest_info(quest_id, npc_id))
                .await?;*/
            }
        }
        // Forfeit quest
        3 => {
            // TODO ensure quest is started
            // TODO if quest time_limit > 0 send remove time limit packet
            session.stream.write_packet(forfeit_quest(quest_id)).await?;
        }
        // Start scripted quest
        4 => {
            log::debug!("Start scripted quest: {}", quest_id);
        }
        // Complete scripted quest
        5 => {
            log::debug!("Complete scripted quest: {}", quest_id);
        }
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

    // Used to update if quest has info_number
    if info_update {
        // packet.write_byte(1);
    } else {
        // TODO this can be QuestStatus as u8, but for initial quest sending a 0 or 2 crashes the client
        // it may be that we have to send the expected status, need to check with other quests
        packet.write_byte(1);
    }

    // TODO string progress data
    packet.write_string("");
    packet.write_bytes(&[0, 0, 0, 0, 0]);
    packet
}

// TODO what does dis do
fn update_quest_info(quest_id: i16, npc_id: i32) -> Packet {
    let mut packet = Packet::new(0xD3);
    packet.write_byte(8);
    packet.write_short(quest_id);
    packet.write_int(npc_id);
    packet.write_int(0);
    packet
}

fn complete_quest(quest_id: i16) -> Packet {
    let mut packet = Packet::new(0x27);
    packet.write_byte(1);
    packet.write_short(quest_id);
    packet.write_byte(2);
    let current_time = Utc::now().timestamp_millis() * 10000;
    let offset: i64 =
        116444736010800000 + (10000000 * i64::from(Local::now().offset().local_minus_utc()));
    packet.write_long(current_time + offset);
    packet
}

//
fn forfeit_quest(quest_id: i16) -> Packet {
    let mut packet = Packet::new(0x27);
    packet.write_byte(1);
    packet.write_short(quest_id);
    packet.write_byte(0);
    packet
}
