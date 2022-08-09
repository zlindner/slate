pub mod net;
pub mod state;
pub mod util;

pub mod character;
pub use self::character::Character;

pub mod db;
pub use self::db::Db;

pub mod redis;
pub use self::redis::Redis;

pub type Error = Box<dyn std::error::Error + Send + Sync>;
pub type Result<T> = std::result::Result<T, Error>;
