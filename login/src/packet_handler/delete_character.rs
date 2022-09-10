use crate::packets::{self, PicOperation};
use deadpool_redis::redis::AsyncCommands;
use oxide_core::{
    net::{Connection, Packet},
    Db, Redis, Result,
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

    pub async fn handle(self, connection: &mut Connection, db: Db, redis: Redis) -> Result<()> {
        let bypass_pic = false;

        if bypass_pic {
            connection.close().await?;
            return Ok(());
        }

        let mut redis = redis.get().await?;
        let key = format!("login_session:{}", connection.session_id);
        let pic_attempts: u8 = redis.hget(&key, "pic_attempts").await?;

        if pic_attempts >= 6 {
            connection.close().await?;
            return Ok(());
        }

        let pic: String = redis.hget(&key, "pic").await?;

        if self.pic != pic {
            redis.hset(&key, "pic_attempts", pic_attempts + 1).await?;

            connection
                .write_packet(packets::delete_character(
                    self.character_id,
                    PicOperation::InvalidPic,
                ))
                .await?;
            return Ok(());
        }

        redis.hset(&key, "pic_attempts", 0).await?;

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

        // TODO need to delete reference to this character in like 10 other tables (buddies, bbs_threads, bbs_replies, wishlists, etc.)

        connection
            .write_packet(packets::delete_character(
                self.character_id,
                PicOperation::Success,
            ))
            .await?;

        Ok(())
    }
}
