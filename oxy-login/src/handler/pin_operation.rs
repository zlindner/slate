use crate::client::LoginClient;
use anyhow::Result;
use oxy_core::net::Packet;

/// Login server: pin operation/after login packet (0x09)
///
pub async fn handle(mut packet: Packet, client: &mut LoginClient) -> Result<()> {
    let a = packet.read_byte();
    let b = match packet.remaining() {
        0 => 5,
        _ => packet.read_byte(),
    };

    let pin = match b {
        0 => packet.read_string(),
        _ => "".to_string(),
    };

    if (a, b) == (1, 1) {
        if client.session.pin.is_empty() {
            let response = pin_operation(PinOperation::Register);
            return client.send(response).await;
        }

        let response = pin_operation(PinOperation::Request);
        return client.send(response).await;
    }

    if (a, b) == (1, 0) || (a, b) == (2, 0) {
        if client.session.pin_attempts >= 6 {
            client.disconnect().await;
            return Ok(());
        }

        client.session.pin_attempts += 1;

        if !client.session.pin.is_empty() && client.session.pin == pin {
            client.session.pin_attempts = 0;

            if a == 1 {
                let response = pin_operation(PinOperation::Accepted);
                return client.send(response).await;
            }

            let response = pin_operation(PinOperation::Register);
            return client.send(response).await;
        }
    }

    let response = pin_operation(PinOperation::Invalid);
    client.send(response).await?;
    Ok(())
}

enum PinOperation {
    Accepted = 0x00,
    Register = 0x01,
    Invalid = 0x02,
    Request = 0x04,
}

fn pin_operation(op: PinOperation) -> Packet {
    let mut packet = Packet::new();
    packet.write_short(0x06);
    packet.write_byte(op as u8);
    packet
}
