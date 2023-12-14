use sqlx::{MySql, Pool};

pub mod config;
pub mod maple;
pub mod nx;
pub mod sql;

pub use self::config::Config;

pub type Db = Pool<MySql>;
