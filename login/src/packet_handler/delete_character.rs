use crate::{
    client::Client,
    packets::{self, PicOperation},
};
use oxide_core::{net::Packet, Db, Result};

pub struct DeleteCharacter {
    pic: String,
    character_id: i32,
}

impl DeleteCharacter {
    pub fn new(mut packet: Packet) -> Self {
        Self {
            pic: packet.read_string(),
            character_id: packet.read_int(),
        }
    }

    pub async fn handle(self, client: &mut Client, db: Db) -> Result<()> {
        let bypass_pic = false; // TODO config option

        if bypass_pic || client.session.pic_attempts >= 6 {
            client.disconnect().await?;
            return Ok(());
        }

        client.session.pic_attempts += 1;

        if self.pic != client.session.pic {
            let packet = packets::delete_character(self.character_id, PicOperation::InvalidPic);
            client.send(packet).await?;
            return Ok(());
        }

        client.session.pic_attempts = 0;

        // TODO check if character is a guild leader
        // TODO check if character has a pending world transfer
        // TODO check if character has a family
        // TODO check if character has a pending wedding

        // TODO check if character is currently in a party
        // if so need to assign new leader (if leader) and remove from party

        // TODO may want to check if the client id matches with character id and is currently logged to prevent exploits

        sqlx::query(
            "DELETE FROM characters \
            WHERE id = $1",
        )
        .bind(self.character_id)
        .execute(&db)
        .await?;

        client.num_characters -= 1;

        // TODO need to delete reference to this character in like 10 other tables (buddies, bbs_threads, bbs_replies, wishlists, etc.)

        let packet = packets::delete_character(self.character_id, PicOperation::Success);
        client.send(packet).await?;
        Ok(())
    }
}
