use crate::{
    model::{Account, LoginState},
    query,
    server::LoginSession,
};
use slime_net::Packet;

/// Login server: login packet (0x01)
/// Called when the client clicks login after entering name and password
pub async fn handle(mut packet: Packet, session: &mut LoginSession) -> anyhow::Result<()> {
    if session.data.login_attempts >= 5 {
        return session
            .stream
            .write_packet(login_failed(LoginError::TooManyAttempts))
            .await;
    }

    session.data.login_attempts += 1;

    let name = packet.read_string();
    let account = sqlx::query_as::<_, Account>("SELECT * FROM accounts WHERE name = ?")
        .bind(name)
        .fetch_optional(&session.db)
        .await?;

    let account = match account {
        Some(account) => account,
        None => {
            return session
                .stream
                .write_packet(login_failed(LoginError::AccountNotFound))
                .await;
        }
    };

    let password = packet.read_string();

    // If a validation error occurred, write a failure packet with the reason
    if let Some(error) = validate_login(session, &account, password) {
        return session.stream.write_packet(login_failed(error)).await;
    };

    packet.skip(6);
    let _hwid = packet.read_bytes(4);
    // TODO do stuff with hwid?

    // Store account data in the session
    session.data.account_id = account.id;
    session.data.pin = account.pin.clone();
    session.data.pic = account.pic.clone();

    // Update the account's login state to LoggedIn
    query::update_login_state(session, LoginState::LoggedIn).await?;

    session
        .stream
        .write_packet(login_succeeded(session, &account))
        .await?;

    Ok(())
}

/// Validates the login attempt
fn validate_login(
    session: &mut LoginSession,
    account: &Account,
    password: String,
) -> Option<LoginError> {
    // Validate the bytes of the entered password against the hash stored in db
    if argon2::verify_encoded(&account.password, password.as_bytes()).is_err() {
        return Some(LoginError::IncorrectPassword);
    }

    // Check if account is already logged in
    if !matches!(account.state, LoginState::LoggedIn) {
        return Some(LoginError::AlreadyLoggedIn);
    }

    // Check if account is banned
    if account.banned {
        return Some(LoginError::AccountBanned);
    }

    // If the account hasn't accepted tos send the accept tos prompt
    if !account.accepted_tos {
        // Set account id so we can fetch the account in the TOS handler
        session.data.account_id = account.id;
        return Some(LoginError::PromptTOS);
    }

    None
}

/// Error returned to client if login fails
enum LoginError {
    AccountBanned = 0x03,
    IncorrectPassword = 0x04,
    AccountNotFound = 0x05,
    TooManyAttempts = 0x06,
    AlreadyLoggedIn = 0x07,
    PromptTOS = 0x17,
}

/// Login failure packet, indicates reason with error code
fn login_failed(error: LoginError) -> Packet {
    let mut packet = Packet::new(0x00);
    packet.write_int(error as i32);
    packet.write_short(0);
    packet
}

/// Login success packet
pub fn login_succeeded(session: &LoginSession, account: &Account) -> Packet {
    let mut packet = Packet::new(0x00);
    packet.write_int(0);
    packet.write_short(0);
    packet.write_int(account.id);
    packet.write_byte(account.gender as u8);
    packet.write_byte(0);
    packet.write_byte(0);
    packet.write_byte(0);
    packet.write_string(&account.name);
    packet.write_byte(0);
    packet.write_byte(0);
    packet.write_long(0);
    packet.write_long(0);
    packet.write_int(1); // "select the world"... TODO test what this does
    packet.write_byte(!session.config.enable_pin as u8); // 0: enabled, 1: disabled

    // 0: register pic (user hasn't registered pic)
    // 1: prompt pic (user already registered pic)
    // 2: disable pic
    let pic_flag = match session.config.enable_pic {
        true => !session.data.pic.is_empty() as u8,
        false => 2,
    };

    packet.write_byte(pic_flag);
    packet
}
