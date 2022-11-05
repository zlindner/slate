use anyhow::Result;
use oxy_core::{
    net::{Client, Packet},
    prisma::account,
};

/// Login server: register pic packet (0x1D)
///
pub async fn handle(mut packet: Packet, client: &mut Client) -> Result<()> {
    if packet.read_byte() == 0 {
        client.disconnect().await;
        return Ok(());
    }

    let character_id = packet.read_int();
    let mac_addr = packet.read_string();
    let host_addr = packet.read_string();
    // TODO we can check mac_addr/hwid from host_addr if we want to prevent multi-logging

    let pic = packet.read_string();

    if pic.is_empty() {
        client.disconnect().await;
        return Ok(());
    }

    // Update account's pic in db
    client
        .db
        .account()
        .update(
            account::id::equals(client.session.account_id),
            vec![account::pic::set(pic.clone())],
        )
        .exec()
        .await?;

    client.session.pic = pic;
    client.session.character_id = character_id;

    super::select_character::connect_to_world_server(client).await?;
    Ok(())
}
