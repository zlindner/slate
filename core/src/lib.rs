pub mod maple;
pub mod net;
pub mod pg;
pub mod util;

pub mod db;
pub use self::db::Db;

pub type Error = Box<dyn std::error::Error + Send + Sync>;
pub type Result<T> = std::result::Result<T, Error>;
