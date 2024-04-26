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

pub struct ClientHandler {
    store: RedisStore,
}

impl ClientHandler {
    pub fn new(store: RedisStore) -> Self {
        Self { store }
    }

    pub fn handle(&mut self, stream: &mut TcpStream) {
        let mut buf = [0; 512];
        while let Ok(n) = stream.read(&mut buf) {
            if n == 0 {
                break;
            }
            let command = BulkString::from_bytes(&buf[..n]);
            match RedisCommands::parse(command) {
                RedisCommands::Ping => self.ping(stream),
                RedisCommands::Echo(message) => self.echo(&message, stream),
                RedisCommands::Unknown => self.unimplemented(stream),
                RedisCommands::Get(key) => self.get(key, stream),
                RedisCommands::Set((key, value)) => self.set(key, value, stream),
            }
        }
    }

    fn unimplemented(&self, stream: &mut TcpStream) {
        self.responde("Not implemented yet", stream)
    }

    fn ping(&self, stream: &mut TcpStream) {
        self.responde("PONG", stream);
    }

    fn echo(&self, message: &[Bulk], stream: &mut TcpStream) {
        let message: Vec<u8> = message.iter().flat_map(|bulk| bulk.to_bytes()).collect();
        self.responde(&Bulk::from_bytes(&message).data(), stream);
    }

    fn set(&self, key: String, value: String, stream: &mut TcpStream) {
        self.store.lock().unwrap().insert(key, value);
        self.responde("OK", stream)
    }

    fn get(&self, key: String, stream: &mut TcpStream) {
        match self.store.lock().unwrap().get(&key) {
            Some(value) => self.responde(value, stream),
            None => self.responde("-1", stream),
        }
    }

    fn responde(&self, data: &str, stream: &mut TcpStream) {
        let response = Bulk::from_data(data).to_bytes();
        stream.write_all(&response).unwrap();
    }
}
