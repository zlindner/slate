use crate::prisma::{account, LoginState, PrismaClient};
use anyhow::Result;
use prisma_client_rust::chrono::{DateTime, FixedOffset, Utc};

/// Updates the login state and last login time (to now) for the given account id.
pub async fn update_login_state(
    db: &PrismaClient,
    account_id: i32,
    new_state: LoginState,
) -> Result<()> {
    if account_id == -1 {
        return Ok(());
    }

    let now: DateTime<FixedOffset> = DateTime::from(Utc::now());

    db.account()
        .update(
            account::id::equals(account_id),
            vec![
                account::state::set(new_state),
                account::last_login::set(Some(now)),
            ],
        )
        .exec()
        .await?;

    Ok(())
}
