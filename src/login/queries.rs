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

pub async fn update_pin(id: i32, pin: &String, db: &Db) -> Result<()> {
    sqlx::query(
        "UPDATE accounts \
        SET pin = $1 \
        WHERE id = $2",
    )
    .bind(pin)
    .bind(id)
    .execute(db)
    .await?;

    Ok(())
}

pub async fn get_characters(client_id: i32, world_id: i32, db: &Db) -> Result<Vec<PgRow>> {
    let res = sqlx::query(
        "SELECT * \
        FROM characters \
        WHERE account_id = $1 AND world_id = $2",
    )
    .bind(client_id)
    .bind(world_id)
    .fetch_all(db)
    .await?;

    Ok(res)
}

pub async fn logout_all(db: &Db) -> Result<()> {
    sqlx::query(
        "UPDATE accounts \
        SET login_state = 0, last_login = CURRENT_TIMESTAMP \
        where login_state = 1",
    )
    .execute(db)
    .await?;

    Ok(())
}
