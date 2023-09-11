use slime_net::MapleStream;
use sqlx::{MySql, Pool};

pub struct WorldServer {}

impl WorldServer {
    pub fn new() -> Self {
        Self {}
    }
}

pub struct WorldSession {
    pub id: i32,
    pub stream: MapleStream,
    pub db: Pool<MySql>,
}
