use crate::Db;
use sqlx::{
    types::chrono::{DateTime, Utc},
    Decode, Encode, FromRow,
};

#[derive(FromRow)]
pub struct Account {
    pub id: i32,
    pub name: String,
    pub password: String,
    pub pin: String,
    pub pic: String,
    pub state: LoginState,
    pub banned: bool,
    pub accepted_tos: bool,
    pub last_login: Option<DateTime<Utc>>,
    pub gender: i32,
}

impl Account {
    /// Loads an account by name if it exists
    pub async fn load_optional_by_name(name: String, db: &Db) -> anyhow::Result<Option<Self>> {
        let account = sqlx::query_as::<_, Account>("SELECT * FROM accounts WHERE name = ?")
            .bind(name)
            .fetch_optional(db)
            .await?;

        Ok(account)
    }

    /// Loads an account by id if it exists
    pub async fn load_optional_by_id(id: i32, db: &Db) -> anyhow::Result<Option<Self>> {
        let account = sqlx::query_as::<_, Account>("SELECT * FROM accounts WHERE id = ?")
            .bind(id)
            .fetch_optional(db)
            .await?;

        Ok(account)
    }

    /// Updates an account's login state
    pub async fn update_login_state(
        account_id: i32,
        new_state: LoginState,
        db: &Db,
    ) -> anyhow::Result<()> {
        sqlx::query("UPDATE accounts SET state = ? WHERE id = ?")
            .bind(new_state)
            .bind(account_id)
            .execute(db)
            .await?;

        Ok(())
    }

    /// Updates an account's PIC
    pub async fn update_pic(account_id: i32, pic: &String, db: &Db) -> anyhow::Result<()> {
        sqlx::query("UPDATE accounts SET pic = ? WHERE id = ?")
            .bind(pic)
            .bind(account_id)
            .execute(db)
            .await?;

        Ok(())
    }

    /// Updates an account's TOS
    pub async fn update_tos(account_id: i32, tos: bool, db: &Db) -> anyhow::Result<()> {
        sqlx::query("UPDATE accounts SET tos = ? WHERE id = ?")
            .bind(tos)
            .bind(account_id)
            .execute(db)
            .await?;

        Ok(())
    }
}

#[derive(Decode, Encode)]
pub enum LoginState {
    LoggedIn,
    Transitioning,
    LoggedOut,
}

impl sqlx::Type<sqlx::MySql> for LoginState {
    fn type_info() -> <sqlx::MySql as sqlx::Database>::TypeInfo {
        <str as sqlx::Type<sqlx::MySql>>::type_info()
    }

    fn compatible(ty: &<sqlx::MySql as sqlx::Database>::TypeInfo) -> bool {
        <str as sqlx::Type<sqlx::MySql>>::compatible(ty)
    }
}
