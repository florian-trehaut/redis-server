use std::{
    collections::HashMap,
    io::{Read, Write},
    net::TcpStream,
    sync::{Arc, Mutex},
};

use resp::{Bulk, BulkString};
mod resp;
enum RedisCommands {
    Ping,
    Echo(Vec<Bulk>),
    Get(String),
    Set((String, String)),
    Unknown,
}
pub type RedisStore = Arc<Mutex<HashMap<String, String>>>;
impl RedisCommands {
    fn parse(bulkstring: BulkString) -> RedisCommands {
        match bulkstring.bulks().first().unwrap().data().as_str() {
            "ping" => RedisCommands::Ping,
            "echo" => RedisCommands::Echo(bulkstring.bulks()[1..].to_vec()),
            "get" => RedisCommands::Get(bulkstring.bulks().get(1).unwrap().data()),
            "set" => RedisCommands::Set((
                bulkstring.bulks().get(1).unwrap().data(),
                bulkstring.bulks().get(2).unwrap().data(),
            )),
            _ => RedisCommands::Unknown,
        }
    }
}

pub fn handle_client(stream: &mut TcpStream, store: RedisStore) {
    let mut buf = [0; 512];
    while let Ok(n) = stream.read(&mut buf) {
        if n == 0 {
            break;
        }
        let command = BulkString::from_bytes(&buf[..n]);
        match RedisCommands::parse(command) {
            RedisCommands::Ping => ping(stream),
            RedisCommands::Echo(message) => echo(stream, &message),
            RedisCommands::Unknown => unimplemented(stream),
            RedisCommands::Get(key) => get(stream, key, &store),
            RedisCommands::Set((key, value)) => set(stream, key, value, &store),
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
    let message: Vec<u8> = message.iter().flat_map(|bulk| bulk.to_bytes()).collect();
    stream
        .write_all(&Bulk::from_bytes(&message).to_bytes())
        .expect("Can't write response to stream");
}

fn set(stream: &mut TcpStream, key: String, value: String, store: &RedisStore) {
    store.lock().unwrap().insert(key, value);
    responde(stream, "OK")
}
fn get(stream: &mut TcpStream, key: String, store: &RedisStore) {
    match store.lock().unwrap().get(&key) {
        Some(value) => responde(stream, value),
        None => responde(stream, NULL),
    }
}
fn responde(stream: &mut TcpStream, data: &str) {
    let response = Bulk::from_data(data).to_bytes();
    stream.write_all(&response).unwrap();
}

const NULL: &str = "-1";
