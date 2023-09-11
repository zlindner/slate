use crate::server::LoginSession;
use slime_net::Packet;

/// Login server: pin operation/after login packet (0x09)
/// TODO
pub async fn handle(mut packet: Packet, session: &mut LoginSession) -> anyhow::Result<()> {
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
        if session.data.pin.is_empty() {
            session
                .stream
                .write_packet(pin_operation(PinOperation::Register))
                .await?;

            return Ok(());
        }

        session
            .stream
            .write_packet(pin_operation(PinOperation::Request))
            .await?;

        return Ok(());
    }

    if (a, b) == (1, 0) || (a, b) == (2, 0) {
        if session.data.pin_attempts >= 6 {
            session.stream.close().await?;
            return Ok(());
        }

        session.data.pin_attempts += 1;

        if !session.data.pin.is_empty() && session.data.pin == pin {
            session.data.pin_attempts = 0;

            if a == 1 {
                session
                    .stream
                    .write_packet(pin_operation(PinOperation::Accepted))
                    .await;

                return Ok(());
            }

            session
                .stream
                .write_packet(pin_operation(PinOperation::Register))
                .await;

            return Ok(());
        }
    }

    session
        .stream
        .write_packet(pin_operation(PinOperation::Invalid))
        .await?;

    Ok(())
}

enum PinOperation {
    Accepted = 0x00,
    Register = 0x01,
    Invalid = 0x02,
    Request = 0x04,
}

/// Packet indicating a pin operation
fn pin_operation(op: PinOperation) -> Packet {
    let mut packet = Packet::new(0x06);
    packet.write_byte(op as u8);
    packet
}
