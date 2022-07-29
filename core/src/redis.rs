use super::Result;
use deadpool_redis::{Config, Pool, Runtime};
use std::env;

pub type Redis = Pool;

pub fn new() -> Result<Redis> {
    let cfg = Config::from_url(&env::var("REDIS_URL").unwrap());
    let pool = cfg.create_pool(Some(Runtime::Tokio1))?;
    Ok(pool)
}
