use crate::{
    client::Client,
    config::CONFIG,
    login::{
        packets::{self, PicOperation},
        queries,
    },
    net::packet::Packet,
    Result,
};

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

    pub async fn handle(self, client: &mut Client) -> Result<()> {
        let op = match Self::validate_pic(client, &self.pic).await? {
            Some(valid) => match valid {
                true => {
                    // TODO check if character is a guild leader
                    // TODO check if character has a pending world transfer
                    // TODO check if character has a family
                    // TODO check if character has a pending wedding

                    // TODO check if character is currently in a party
                    // if so need to assign new leader (if leader) and remove from party

                    // TODO may want to check if the client id matches with character id and is currently logged to prevent exploits

                    queries::delete_character(self.character_id, &client.db).await?;
                    Some(PicOperation::Success)
                }
                false => Some(PicOperation::InvalidPic),
            },
            None => None,
        };

        if op.is_some() {
            client
                .connection
                .write_packet(packets::delete_character(self.character_id, op.unwrap()))
                .await?;
        }

        Ok(())
    }

    async fn validate_pic(client: &mut Client, pic: &String) -> Result<Option<bool>> {
        if CONFIG.bypass_pic {
            return Ok(Some(true));
        }

        client.pic_attempts += 1;

        if client.pic_attempts >= 6 {
            client.disconnect().await?;
            return Ok(None);
        }

        if client.pic.is_some() && pic == client.pic.as_ref().unwrap() {
            client.pic_attempts = 0;
            return Ok(Some(true));
        }

        Ok(Some(false))
    }
}
