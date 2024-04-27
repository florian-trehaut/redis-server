use std::{
    cmp::Ordering,
    io::{Read, Write},
    net::TcpStream,
    time::{Duration, Instant},
};

use crate::{
    redis_commands::RedisCommands,
    redis_info::RedisInfo,
    resp::{Bulk, BulkString, RedisResponse, ToRedisBytes},
    RedisStore, RedisValue,
};

pub struct ClientHandler {
    store: RedisStore,
    server_info: RedisInfo,
}

impl ClientHandler {
    pub fn new(store: RedisStore) -> Self {
        let server_info = RedisInfo::new();
        Self { store, server_info }
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
                RedisCommands::Set((key, value, expiration)) => {
                    self.set(key, value, expiration, stream)
                }
                RedisCommands::Info(section) => self.info(section, stream),
            }
        }
    }

    fn unimplemented(&self, stream: &mut TcpStream) {
        self.responde(RedisResponse::Unimplemented, stream)
    }

    fn ping(&self, stream: &mut TcpStream) {
        self.responde(RedisResponse::Pong, stream);
    }

    fn echo(&self, message: &[Bulk], stream: &mut TcpStream) {
        let message: Vec<u8> = message
            .iter()
            .flat_map(|bulk| bulk.to_redis_bytes())
            .collect();
        self.responde(Bulk::from_bytes(&message), stream);
    }

    fn set(
        &self,
        key: String,
        value: String,
        expiration: Option<Duration>,
        stream: &mut TcpStream,
    ) {
        let value = RedisValue::new(value, expiration);
        match self
            .store
            .lock()
            .unwrap()
            .insert(key.clone(), value.clone())
        {
            Some(redis_value) => println!(
                "Set -- Successfully updated key:{} value:{} with expiration: {:?}",
                key,
                redis_value.value(),
                redis_value.expiration()
            ),
            None => println!(
                "Set -- Successfully inserted key:{} value:{} with expiration: {:?}",
                key,
                value.value(),
                value.expiration()
            ),
        }
        self.responde(RedisResponse::Ok, stream)
    }

    fn get(&self, key: String, stream: &mut TcpStream) {
        let store = self.store.lock().unwrap();
        let redis_value = match store.get(&key) {
            Some(redis_value) => redis_value,
            None => {
                println!("Get -- Key:{key} has not been found");
                self.responde(RedisResponse::Null, stream);
                return;
            }
        };
        let expiration = match redis_value.expiration() {
            Some(expiration) => expiration,
            None => {
                println!("Get -- Key:{key} has been found and have no expiration");
                self.responde(redis_value.value().clone(), stream);
                return;
            }
        };

        match Instant::now().cmp(&expiration) {
            Ordering::Equal | Ordering::Less => {
                println!("Get -- Key:{key} has been found and is not expired");
                self.responde(redis_value.value().clone(), stream)
            }
            Ordering::Greater => {
                println!("Get -- Key:{key} has been found but is expired");
                self.responde(RedisResponse::Null, stream)
            }
        }
    }

    fn responde(&self, response: impl ToRedisBytes, stream: &mut TcpStream) {
        println!(
            "Responding with: {:?}",
            std::str::from_utf8(&response.to_redis_bytes())
        );
        let response = response.to_redis_bytes();
        stream.write_all(&response).unwrap();
    }

    fn info(&self, section: String, stream: &mut TcpStream) {
        let info = match section.to_lowercase().as_str() {
            "replication" => self.server_info.to_bulk_string(),
            _ => Bulk::from_string("Unknown section"),
        };
        self.responde(info, stream);
    }
}
