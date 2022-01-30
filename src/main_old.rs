use simple_logger::SimpleLogger;
use std::io::{ErrorKind, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::thread;
mod maple_aes;
use crate::maple_aes::maple_aes::MapleAES;

#[derive(Debug)]
struct Packet {
    data: Vec<u8>,
}

impl Packet {
    fn new(bytes: usize) -> Self {
        Packet {
            data: Vec::with_capacity(bytes),
        }
    }

    fn write_byte(&mut self, byte: u8) {
        self.data.push(byte);
    }

    fn write_bytes(&mut self, bytes: &[u8]) {
        self.data.extend_from_slice(bytes);
    }

    // TODO write boolean

    // writes a short (signed 16 bit integer) to the packet
    fn write_short(&mut self, num: i16) {
        let bytes = num.to_le_bytes();
        self.data.extend_from_slice(&bytes);
    }

    // TODO write int
    // TODO write long
    // TODO write string

    fn write_maple_string(&mut self, str: &str) {
        // write the string length as an i16/short
        self.write_short(str.len() as i16);

        let bytes = str.as_bytes();
        self.data.extend_from_slice(&bytes);
    }
}

trait PacketWriter {
    fn write_packet(&mut self, packet: Packet);
}

impl PacketWriter for TcpStream {
    fn write_packet(&mut self, packet: Packet) {
        self.write_all(&packet.data).unwrap();
        self.flush().unwrap();

        log::debug!("write_packet: {:?}", packet.data);
    }
}

trait PacketReader {
    fn read_packet(&mut self, bytes: usize) -> Packet;
}

impl PacketReader for TcpStream {
    fn read_packet(&mut self, bytes: usize) -> Packet {
        let mut buf = Vec::with_capacity(bytes);
        self.take(bytes as u64).read_to_end(&mut buf).unwrap();

        log::debug!("read_packet: {:?}", buf);

        Packet { data: buf }
    }
}

struct Session {
    stream: TcpStream,
    id: usize,
    cypher_receive: MapleAES,
    cypher_send: MapleAES,
}

impl Session {
    fn new(stream: TcpStream, id: usize, cypher_receive: MapleAES, cypher_send: MapleAES) -> Self {
        Session {
            stream,
            id,
            cypher_receive,
            cypher_send,
        }
    }
}

static SESSION_ID: AtomicUsize = AtomicUsize::new(0);

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
                    handle_connection(stream, SESSION_ID.fetch_add(1, Ordering::Relaxed));
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

fn handle_connection(stream: TcpStream, session_id: usize) {
    stream.set_nodelay(true).unwrap();

    log::info!(
        "Client connected to login server: session {} @ {}",
        session_id,
        stream.peer_addr().unwrap()
    );

    // create the cypher for receiving packets
    let iv_receive: [u8; 4] = [70, 114, 122, rand::random::<u8>()];
    let cypher_receive = MapleAES::new(iv_receive, 0xffff - 83);

    // create the cypher for sending packets
    let iv_send: [u8; 4] = [82, 48, 120, rand::random::<u8>()];
    let cypher_send = MapleAES::new(iv_receive, 83);

    // create an encrypted session
    let mut session = Session::new(stream, session_id, cypher_receive, cypher_send);

    // write an unencrypted login handshake packet
    session
        .stream
        .write_packet(login_handshake(iv_receive, iv_send));

    //stream.read_packet(6);
    // [41, 216, 43, 216, 28, 175]

    // TODO get raw packet dump, this is already converted to hex I think
    // <UnknownPacket> ClientSend:null [23] (2) <HEX> 23 00 <TEXT> #.
    // ServerSend:PING [11] (2) <HEX> 11 00 <TEXT> ..
    // ClientSend:PONG [18] (2) <HEX> 18 00 <TEXT> ..
    // ClientSend:LOGIN_PASSWORD [1] (54) <HEX> 01 00 09 00 6A 6A 6A 66 64 73 61 6A 6B 0C 00 66 6A 64 6B 73 61 6C 6A 66 6C 64 6B 00 00 00 00 00 00 35 C7 3F 04 00 00 00 00 F7 C8 00 00 00 00 02 00 00 00 00 00 00 <TEXT> ....jjjfdsajk..fjdksaljfldk......5�?.....��...........
}

fn login_handshake(iv_receive: [u8; 4], iv_send: [u8; 4]) -> Packet {
    let mut packet = Packet::new(18);
    packet.write_short(14); // packet length (0x0E)
    packet.write_short(83); // maple version (v83)
    packet.write_maple_string("1"); // maple patch version (1)
    packet.write_bytes(&iv_receive);
    packet.write_bytes(&iv_send);
    packet.write_byte(8); // locale
    packet
}
