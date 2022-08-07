use deadpool_redis::redis::AsyncCommands;
use oxide_core::{Redis, Result};

#[derive(Debug)]
pub struct Session {
    pub id: i32,
    pub account_id: i32,
    pub pin: Option<String>,
    pub pin_attempts: i32,
    pub pic: Option<String>,
    pub pic_attempts: i32,
    pub login_attempts: i32,
}

impl Session {
    pub async fn create(id: i32, redis: &Redis) -> Result<()> {
        let mut state = redis.get().await?;
        let key = format!("session:{}", id);

        state
            .hset_multiple(
                key,
                &[
                    ("id", id.to_string()),
                    ("account_id", "-1".to_string()),
                    ("pin", String::new()),
                    ("pin_attempts", "0".to_string()),
                    ("pic", String::new()),
                    ("pic_attempts", String::new()),
                    ("login_attempts", "0".to_string()),
                ],
            )
            .await?;

        Ok(())
    }

    pub async fn get(id: i32, redis: Redis) -> Result<Self> {
        let mut state = redis.get().await?;
        let key = format!("session:{}", id);

        let hm = state.hgetall(key).await?;

        // FIXME parse hm
        let session = Session {
            id,
            account_id: -1,
            pin: None,
            pin_attempts: 0,
            pic: None,
            pic_attempts: 0,
            login_attempts: 0,
        };

        Ok(session)
    }
}
