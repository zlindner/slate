use sqlx::{MySql, Pool};

pub mod maple;
pub mod nx;
pub mod sql;

pub type Db = Pool<MySql>;
