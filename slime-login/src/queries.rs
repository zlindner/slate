use crate::{model::LoginState, server::LoginSession};
use anyhow::Result;
use sqlx::types::chrono::Utc;

/// Updates an account's login state and last login time
pub async fn update_login_state(session: &LoginSession, new_state: LoginState) -> Result<()> {
    sqlx::query("UPDATE accounts SET state = ?, last_login = ? WHERE account.id = ?")
        .bind(new_state)
        .bind(Utc::now())
        .bind(session.data.account_id)
        .execute(&session.db)
        .await?;

    Ok(())
}
