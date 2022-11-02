use anyhow::Result;
use oxy_core::{
    net::{Client, Packet},
    prisma::{account, LoginState},
};

/// Login server: register pin packet (0x0A)
///
pub async fn handle(mut packet: Packet, client: &mut Client) -> Result<()> {
    if packet.read_byte() == 0 {
        client.disconnect().await;
        return Ok(());
    }

    let pin = packet.read_string();

    if pin.is_empty() {
        client.disconnect().await;
        return Ok(());
    }

    // Update account's pin in db
    client
        .db
        .account()
        .update(
            account::id::equals(client.session.account_id),
            vec![account::pin::set(pin.clone())],
        )
        .exec()
        .await?;

    client.session.pin = pin;
    client.update_state(LoginState::LoggedOut).await?;

    let response = pin_registered();
    client.send(response).await?;
    Ok(())
}

///
fn pin_registered() -> Packet {
    let mut packet = Packet::new();
    packet.write_short(0x07);
    packet.write_byte(0);
    packet
}
