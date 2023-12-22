use crate::server::LoginSession;
use slate_net::Packet;

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

    let operation = match (a, b) {
        (1, 1) => {
            if session.data.pin.is_empty() {
                PinOperation::Register
            } else {
                PinOperation::Request
            }
        }
        (1, 0) | (2, 0) => {
            if session.data.pin_attempts >= 6 {
                return session.stream.close().await;
            }

            session.data.pin_attempts += 1;

            if !session.data.pin.is_empty() && session.data.pin == pin {
                session.data.pin_attempts = 0;

                if a == 1 {
                    PinOperation::Accepted
                } else {
                    PinOperation::Register
                }
            } else {
                PinOperation::Invalid
            }
        }
        _ => PinOperation::Invalid,
    };

    session
        .stream
        .write_packet(pin_operation(operation))
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
