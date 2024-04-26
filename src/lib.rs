use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};

use resp::{Bulk, BulkString};
pub mod client_handler;
mod resp;
enum RedisCommands {
    Ping,
    Echo(Vec<Bulk>),
    Get(String),
    Set((String, String, Option<Duration>)),
    Unknown,
}
#[derive(Clone)]
pub struct RedisValue {
    value: String,
    expiration: Option<Instant>,
}
impl RedisValue {
    pub fn new(value: String, expiration: Option<Duration>) -> RedisValue {
        let expiration = expiration.map(|expiration| Instant::now() + expiration);
        RedisValue { value, expiration }
    }
}
pub type RedisStore = Arc<Mutex<HashMap<String, RedisValue>>>;
impl RedisCommands {
    fn parse(bulkstring: BulkString) -> RedisCommands {
        match bulkstring.bulks().first().unwrap().data().as_str() {
            "ping" => RedisCommands::Ping,
            "echo" => RedisCommands::Echo(bulkstring.bulks()[1..].to_vec()),
            "get" => RedisCommands::Get(bulkstring.bulks().get(1).unwrap().data()),
            "set" => {
                let bulks = bulkstring.bulks();
                match bulks.as_slice() {
                    [_, key, value, command, duration, ..] if command.data().as_str() == "px" => {
                        let duration = Duration::from_millis(duration.data().parse().unwrap_or(0));
                        RedisCommands::Set((key.data(), value.data(), Some(duration)))
                    }
                    [_, key, value, ..] => RedisCommands::Set((key.data(), value.data(), None)),
                    _ => RedisCommands::Unknown,
                }
            }
            _ => RedisCommands::Unknown,
        }
    }
}
