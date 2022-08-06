pub mod net;
pub mod util;

pub(crate) mod character;
pub use self::character::Character;

pub mod db;
pub use self::db::Db;

pub type Error = Box<dyn std::error::Error + Send + Sync>;
pub type Result<T> = std::result::Result<T, Error>;
