use crate::{character::Character, db::Db, Result};
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

pub async fn create_character(character: &Character, db: &Db) -> Result<()> {
    sqlx::query(
        "INSERT INTO characters \
        (account_id, world_id, name, level, str, dex, luk, int, hp, mp, max_hp, max_mp, mesos, job, skin_colour, gender, hair, face, ap, sp, map, spawn_point, gm) \
        VALUES($1, $2, $3, $4, $5, $6, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18, $19, $20, $21, $22, $23)",
    )
    .bind(character.account_id)
    .bind(character.world_id)
    .bind(character.name.clone())
    .bind(character.stats.level)
    .bind(character.stats.str)
    .bind(character.stats.dex)
    .bind(character.stats.luk)
    .bind(character.stats.int)
    .bind(character.stats.hp)
    .bind(character.stats.mp)
    .bind(character.stats.max_hp)
    .bind(character.stats.max_mp)
    .bind(character.stats.mesos)
    .bind(character.job)
    .bind(character.style.skin_colour)
    .bind(character.style.gender as i8)
    .bind(character.style.hair)
    .bind(character.style.face)
    .bind(character.stats.ap)
    .bind(character.stats.sp.clone())
    .bind(character.map)
    .bind(character.spawn_point)
    .bind(character.gm as i8)
    .execute(db)
    .await?;

    Ok(())
}

pub async fn get_character_id_by_name(name: &String, db: &Db) -> Result<Option<PgRow>> {
    let res = sqlx::query(
        "SELECT id \
        FROM characters \
        WHERE name = $1",
    )
    .bind(name)
    .fetch_optional(db)
    .await?;

    Ok(res)
}
