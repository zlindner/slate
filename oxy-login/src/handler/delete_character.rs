use crate::{client::LoginClient, shared::Shared};
use anyhow::Result;
use oxy_core::{net::Packet, prisma::character};

/// Login server: delete character packet (0x17)
///
pub async fn handle(mut packet: Packet, client: &mut LoginClient, shared: &Shared) -> Result<()> {
    let pic = packet.read_string();

    if shared.config.enable_pic {
        if client.session.pic_attempts >= 6 {
            client.disconnect().await;
            return Ok(());
        }

        client.session.pic_attempts += 1;

        if client.session.pic.is_empty() || client.session.pic != pic {
            let response = super::select_character_pic::invalid_pic();
            return client.send(response).await;
        }
    }

    let character_id = packet.read_int();
    let character = client
        .db
        .character()
        .find_unique(character::id::equals(character_id))
        .exec()
        .await?;

    let character = match character {
        Some(character) => character,
        None => {
            let response = delete_character(character_id, Reason::Unknown);
            return client.send(response).await;
        }
    };

    // TODO if character is a guild leader send DeleteCharacterError::GuildMaster
    // TODO if character has a pending wedding send DeleteCharacterError::PendingWedding
    // TODO if character has a pending world transfer send DeleteCharacterError::PendingWorldTransfer
    // TODO if character has a family send DeleteCharacterError::FamilyMember

    // TODO handle if player is currently in a party

    // Delete the character (and all relations) from db
    // NOTE: relations to Character should be created with `onDelete: Cascade`
    // so they are automatically deleted here
    client
        .db
        .character()
        .delete(character::id::equals(character_id))
        .exec()
        .await?;

    let response = delete_character(character_id, Reason::Success);
    client.send(response).await?;
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

fn delete_character(character_id: i32, reason: Reason) -> Packet {
    let mut packet = Packet::new();
    packet.write_short(0x0F);
    packet.write_int(character_id);
    packet.write_byte(reason as u8);
    packet
}
