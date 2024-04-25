use std::{
    io::{Read, Write},
    net::TcpStream,
};

use resp::{Bulk, BulkString};
mod resp;
enum RedisCommands {
    PING,
    ECHO(Vec<Bulk>),
    UNKOWN,
}
impl RedisCommands {
    fn parse(bulkstring: BulkString) -> RedisCommands {
        match bulkstring.bulks().first().unwrap().data().as_str() {
            "ping" => RedisCommands::PING,
            "echo" => RedisCommands::ECHO(bulkstring.bulks()[1..].to_vec()),
            _ => RedisCommands::UNKOWN,
        }
    }
}

pub fn handle_client(stream: &mut TcpStream) {
    let mut buf = [0; 512];
    while let Ok(n) = stream.read(&mut buf) {
        if n == 0 {
            break;
        }
        let command = BulkString::from_bytes(&buf[..n]);
        match RedisCommands::parse(command) {
            RedisCommands::PING => ping(stream),
            RedisCommands::ECHO(message) => echo(stream, &message),
            RedisCommands::UNKOWN => unimplemented(stream),
        }
    }
}
fn unimplemented(stream: &mut TcpStream) {
    stream
        .write_all(&Bulk::from_data("Not implemented yet").to_bytes())
        .expect("Can't write to stream")
}
fn ping(stream: &mut TcpStream) {
    stream
        .write_all(&Bulk::from_data("PONG").to_bytes())
        .expect("Can't write PONG to stream");
}
fn echo(stream: &mut TcpStream, message: &[Bulk]) {
    let message: Vec<u8> = message
        .iter()
        .flat_map(|bulk| bulk.to_bytes())
        .collect();
    stream
        .write_all(&Bulk::from_bytes(&message).to_bytes())
        .expect("Can't write response to stream");
}
