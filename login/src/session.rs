use deadpool_redis::redis::AsyncCommands;
use oxide_core::{Redis, Result};

#[derive(Debug, Default)]
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
        let mut session = Session::default();
        session.id = id;
        session.save(redis).await?;
        Ok(())
    }

    pub async fn delete(id: i32, redis: &Redis) -> Result<()> {
        let mut state = redis.get().await?;
        let key = format!("session:{}", id);
        state.del(key).await?;
        Ok(())
    }

    pub async fn load(id: i32, redis: &Redis) -> Result<Self> {
        let mut state = redis.get().await?;
        let key = format!("session:{}", id);

        let hm = state.hgetall(key).await?;
        log::debug!("hm: {:?}", hm);

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

    pub async fn save(self, redis: &Redis) -> Result<()> {
        let mut state = redis.get().await?;
        let key = format!("session:{}", self.id);

        state
            .hset_multiple(
                key,
                &[
                    ("id", self.id.to_string()),
                    ("account_id", self.account_id.to_string()),
                    ("pin", self.pin.unwrap_or(String::new())),
                    ("pin_attempts", self.pin_attempts.to_string()),
                    ("pic", self.pic.unwrap_or(String::new())),
                    ("pic_attempts", self.pic_attempts.to_string()),
                    ("login_attempts", self.login_attempts.to_string()),
                ],
            )
            .await?;

        Ok(())
    }
}
