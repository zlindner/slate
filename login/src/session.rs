use deadpool_redis::redis::AsyncCommands;
use oxide_core::{Redis, Result};
use std::collections::HashMap;

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

        let map: HashMap<String, String> = state.hgetall(key).await?;
        let pin = map.get("pin").unwrap();
        let pin = (!pin.is_empty()).then(|| pin.to_owned());
        let pic = map.get("pic").unwrap();
        let pic = (!pic.is_empty()).then(|| pic.to_owned());

        let session = Session {
            id,
            account_id: map.get("account_id").unwrap().parse().unwrap(),
            pin,
            pin_attempts: map.get("pin_attempts").unwrap().parse().unwrap(),
            pic,
            pic_attempts: map.get("pic_attempts").unwrap().parse().unwrap(),
            login_attempts: map.get("login_attempts").unwrap().parse().unwrap(),
        };

        log::debug!("loaded session: {:?}", session);
        Ok(session)
    }

    pub async fn save(&self, redis: &Redis) -> Result<()> {
        let mut state = redis.get().await?;
        let key = format!("session:{}", self.id);

        // TODO look into creating macro? struct -> hashmap?
        state
            .hset_multiple(
                key,
                &[
                    ("id", self.id.to_string()),
                    ("account_id", self.account_id.to_string()),
                    ("pin", self.pin.to_owned().unwrap_or_default()),
                    ("pin_attempts", self.pin_attempts.to_string()),
                    ("pic", self.pic.to_owned().unwrap_or_default()),
                    ("pic_attempts", self.pic_attempts.to_string()),
                    ("login_attempts", self.login_attempts.to_string()),
                ],
            )
            .await?;

        Ok(())
    }
}
