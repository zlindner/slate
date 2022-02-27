use crate::{db::Db, Result};

use sqlx::postgres::PgRow;

pub async fn get_account(name: &String, db: &Db) -> Result<PgRow> {
    let res = sqlx::query(
        "SELECT id, name, password, pin, pic, login_state, last_login, banned, accepted_tos \
        FROM accounts \
        WHERE name = $1",
    )
    .bind(name)
    .fetch_one(db)
    .await?;

    Ok(res)
}

pub async fn update_login_state(id: i32, login_state: i16, db: &Db) -> Result<()> {
    sqlx::query(
        "UPDATE accounts \
        SET login_state = $1, last_login = CURRENT_TIMESTAMP \
        WHERE id = $2",
    )
    .bind(login_state)
    .bind(id)
    .execute(db)
    .await?;

    Ok(())
}
