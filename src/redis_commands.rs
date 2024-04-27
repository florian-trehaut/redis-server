use std::time::Duration;

use crate::resp::{Bulk, BulkString};

pub enum RedisCommands {
    Ping,
    Echo(Vec<Bulk>),
    Get(String),
    Set((String, String, Option<Duration>)),
    Unknown,
}
impl RedisCommands {
    pub fn parse(bulkstring: BulkString) -> RedisCommands {
        match bulkstring
            .bulks()
            .first()
            .unwrap()
            .data()
            .to_lowercase()
            .as_str()
        {
            "ping" => RedisCommands::Ping,
            "echo" => RedisCommands::Echo(bulkstring.bulks()[1..].to_vec()),
            "get" => RedisCommands::Get(bulkstring.bulks().get(1).unwrap().data()),
            "set" => {
                let bulks = bulkstring.bulks();
                match bulks.as_slice() {
                    [_, key, value, command, duration, ..]
                        if command.data().to_lowercase().as_str() == "px" =>
                    {
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
