use anyhow::Result;
use oxy_core::{
    net::{Client, Packet},
    prisma::character,
};

/// Login server: delete character packet (0x17)
///
pub async fn handle(mut packet: Packet, client: &mut Client) -> Result<()> {
    let pic = packet.read_string();
    let character_id = packet.read_int();

    // TODO Validate PIC if enabled

    let character = client
        .db
        .character()
        .find_unique(character::id::equals(character_id))
        .exec()
        .await?;

    let character = match character {
        Some(character) => character,
        None => {
            let response = delete_character_error(character_id, DeleteCharacterError::Unknown);
            return client.send(response).await;
        }
    };

    // TODO if character is a guild leader send DeleteCharacterError::GuildMaster
    // TODO if character has a pending wedding send DeleteCharacterError::PendingWedding
    // TODO if character has a pending world transfer send DeleteCharacterError::PendingWorldTransfer
    // TODO if character has a family send DeleteCharacterError::FamilyMember

    // TODO handle if player is currently in a party
    // TODO delete character_id from buddies table
    // TODO delete character_id from bbs_threads table
    // TODO delete character_id from wishlists table
    // TODO delete character_id from buddies table

    // TODO does this delete relations?
    client
        .db
        .character()
        .delete(character::id::equals(character_id))
        .exec()
        .await?;

    let response = delete_character_success(character_id);
    client.send(response).await?;
    Ok(())
}

enum DeleteCharacterError {
    Unknown = 0x09,
    InvalidPic = 0x14,
    GuildMaster = 0x16,
    PendingWedding = 0x18,
    PendingWorldTransfer = 0x1A,
    FamilyMember = 0x1D,
}

fn delete_character_success(character_id: i32) -> Packet {
    let mut packet = Packet::new();
    packet.write_int(character_id);
    packet.write_byte(0);
    packet
}

fn delete_character_error(character_id: i32, reason: DeleteCharacterError) -> Packet {
    let mut packet = Packet::new();
    packet.write_int(character_id);
    packet.write_byte(reason as u8);
    packet
}
