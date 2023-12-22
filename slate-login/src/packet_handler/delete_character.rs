use crate::server::LoginSession;
use slate_data::sql;
use slate_net::Packet;

/// Login server: delete character packet (0x17)
/// Handles character deletion
pub async fn handle(mut packet: Packet, session: &mut LoginSession) -> anyhow::Result<()> {
    let pic = packet.read_string();

    if session.config.enable_pic {
        if session.data.pic_attempts >= 6 {
            return session.stream.close().await;
        }

        session.data.pic_attempts += 1;

        if session.data.pic.is_empty() || session.data.pic != pic {
            return session
                .stream
                .write_packet(super::select_character_pic::invalid_pic())
                .await;
        }
    }

    let character_id = packet.read_int();
    let character = sqlx::query_as::<_, sql::Character>("SELECT * FROM characters WHERE id = ?")
        .bind(character_id)
        .fetch_optional(&session.db)
        .await?;

    let character = match character {
        Some(character) => character,
        None => {
            return session
                .stream
                .write_packet(delete_character(character_id, Reason::Unknown))
                .await;
        }
    };

    // TODO if character is a guild leader send DeleteCharacterError::GuildMaster
    // TODO if character has a pending wedding send DeleteCharacterError::PendingWedding
    // TODO if character has a pending world transfer send DeleteCharacterError::PendingWorldTransfer
    // TODO if character has a family send DeleteCharacterError::FamilyMember
    // TODO handle if character is currently in a party

    // Delete the character from db
    // TODO need to delete all relations (equips, items, keymaps, etc.)
    sqlx::query("DELETE FROM characters WHERE id = ?")
        .bind(character_id)
        .execute(&session.db)
        .await?;

    session
        .stream
        .write_packet(delete_character(character_id, Reason::Success))
        .await?;

    Ok(())
}

enum Reason {
    Success = 0x00,
    Unknown = 0x09,
    InvalidPic = 0x14,
    GuildMaster = 0x16,
    PendingWedding = 0x18,
    PendingWorldTransfer = 0x1A,
    FamilyMember = 0x1D,
}

/// Packat indicating the delete character result
fn delete_character(character_id: i32, reason: Reason) -> Packet {
    let mut packet = Packet::new(0x0F);
    packet.write_int(character_id);
    packet.write_byte(reason as u8);
    packet
}
