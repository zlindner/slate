use deadpool_postgres::Pool;

use crate::packet::Packet;

enum LoginError {
    AccountNotFound,
}

pub async fn login(mut packet: Packet, pool: &Pool) {
    let name = packet.read_maple_string();
    let password = packet.read_maple_string();
    packet.advance(6);
    let hwid = packet.read_bytes(4);

    log::debug!(
        "Attempting to login: [name: {}, password: {}, hwid: {:?}]",
        name,
        password,
        hwid
    );

    get_account(name, pool).await;
}

//
async fn get_account(name: String, pool: &Pool) -> Result<(), LoginError> {
    let client = pool.get().await.unwrap();
    let rows = client
        .query("SELECT password FROM accounts WHERE name = $1", &[&name])
        .await
        .unwrap();

    if rows.len() == 0 {
        log::debug!("Account doesn't exist");
        return Err(LoginError::AccountNotFound);
    }

    // account exists, return account

    Ok(())
}
