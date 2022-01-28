use simple_logger::SimpleLogger;
use std::io::{ErrorKind, Write};
use std::net::{TcpListener, TcpStream};
use std::thread;

#[derive(Debug)]
struct Packet {
    data: Vec<u8>,
}

impl Packet {
    fn new(size: usize) -> Packet {
        Packet {
            data: Vec::with_capacity(size),
        }
    }

    fn write_bytes(&mut self, bytes: &[u8]) {
        self.data.extend_from_slice(bytes);
    }

    fn write_byte(&mut self, byte: u8) {
        self.data.push(byte);
    }

    // writes a short (signed 16 bit integer) to the packet
    fn write_i16(&mut self, num: i16) {
        // convert the num to it's little endian byte representation
        let bytes = num.to_le_bytes();
        self.data.extend_from_slice(&bytes);
    }

    fn write_maple_string(&mut self, str: &str) {
        // write the string length as an i16/short
        self.write_i16(str.len() as i16);

        let bytes = str.as_bytes();
        self.data.extend_from_slice(&bytes);
    }
}

trait PacketHandler {
    fn write_packet(&mut self, packet: Packet);
    //fn read_packet(&self) -> Packet;
}

impl PacketHandler for TcpStream {
    fn write_packet(&mut self, packet: Packet) {
        self.write_all(&packet.data).unwrap();
        //self.flush().unwrap();
    }

    //fn read_packet(&self) -> Packet {}
}

fn main() {
    SimpleLogger::new().env().init().unwrap();

    let listener = TcpListener::bind("127.0.0.1:8484").unwrap();
    listener
        .set_nonblocking(true)
        .expect("Cannot set non-blocking");

    log::info!("Login server started on port 8484");

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                thread::spawn(|| {
                    handle_connection(stream);
                });
            }
            Err(ref e) if e.kind() == ErrorKind::WouldBlock => {
                continue;
            }
            Err(e) => {
                drop(listener);
                panic!("Login server encountered a fatal error: {}", e);
            }
        }
    }
}

fn handle_connection(mut stream: TcpStream) {
    log::info!(
        "Client connected to login server from: {}",
        stream.peer_addr().unwrap()
    );

    let iv_receive: [u8; 4] = [70, 114, 122, rand::random::<u8>()];
    let iv_send: [u8; 4] = [82, 48, 120, rand::random::<u8>()];

    stream.write_packet(get_login_handshake(iv_receive, iv_send));
}

fn get_login_handshake(iv_receive: [u8; 4], iv_send: [u8; 4]) -> Packet {
    let mut packet = Packet::new(18);

    packet.write_i16(14); // packet length (0x0E)
    packet.write_i16(83); // maple version (v83)
    packet.write_maple_string("1"); // maple patch version (1)
    packet.write_bytes(&iv_receive);
    packet.write_bytes(&iv_send);
    packet.write_byte(8); // locale

    println!("{:?}", packet);
    return packet;
}
