use std::time::Duration;

use crate::resp::{BulkString, RespArray, ToRedisBytes};

pub enum RedisCommands {
    Ping,
    Echo(Vec<BulkString>),
    Get(String),
    Set((String, String, Option<Duration>)),
    Info(String),
    Replconf(String, String),
}
impl RedisCommands {
    pub fn parse(bulkstring: RespArray) -> Result<RedisCommands, RedisCommandError> {
        let redis_command = match bulkstring
            .bulkstrings()
            .first()
            .ok_or(RedisCommandError::EmptyCommand)?
            .data()
            .to_lowercase()
            .as_str()
        {
            "ping" => RedisCommands::Ping,
            "echo" => RedisCommands::Echo(bulkstring.bulkstrings()[1..].to_vec()),
            "get" => RedisCommands::Get(
                bulkstring
                    .bulkstrings()
                    .get(1)
                    .ok_or(RedisCommandError::EmptyGetCommand)?
                    .data(),
            ),
            "set" => {
                let bulks = bulkstring.bulkstrings();
                match bulks.as_slice() {
                    [_, key, value, command, duration, ..]
                        if command.data().to_lowercase().as_str() == "px" =>
                    {
                        let duration = duration.data().parse();
                        let duration = match duration {
                            Ok(value) => Duration::from_millis(value),
                            Err(_) => return Err(RedisCommandError::InvalidSetExpiration),
                        };
                        RedisCommands::Set((key.data(), value.data(), Some(duration)))
                    }
                    [_, key, value, ..] => RedisCommands::Set((key.data(), value.data(), None)),
                    _ => return Err(RedisCommandError::EmptySetKeyOrValue),
                }
            }
            "info" => {
                let section = bulkstring.bulkstrings().get(1).map(|bulk| bulk.data());
                RedisCommands::Info(section.ok_or(RedisCommandError::EmptyInfoSection)?)
            }
            "replconf" => {
                let (command, value) = (
                    bulkstring.bulkstrings().get(1).map(|bulk| bulk.data()),
                    bulkstring.bulkstrings().get(2).map(|bulk| bulk.data()),
                );
                RedisCommands::Replconf(
                    command.ok_or(RedisCommandError::EmptyReplConfCommand)?,
                    value.ok_or(RedisCommandError::EmptyReplConfValue)?,
                )
            }
            _ => return Err(RedisCommandError::InvalidCommand),
        };
        Ok(redis_command)
    }
}

#[derive(Debug)]
pub enum RedisCommandError {
    InvalidCommand,
    EmptyCommand,
    EmptyGetCommand,
    InvalidSetExpiration,
    EmptyInfoSection,
    EmptySetKeyOrValue,
    EmptyReplConfCommand,
    EmptyReplConfValue,
}
impl std::fmt::Display for RedisCommandError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RedisCommandError::InvalidCommand => write!(f, "Invalid command"),
            RedisCommandError::EmptyCommand => write!(f, "Empty command"),
            RedisCommandError::EmptyGetCommand => write!(f, "Empty get command"),
            RedisCommandError::InvalidSetExpiration => write!(f, "Invalid set expiration"),
            RedisCommandError::EmptyInfoSection => write!(f, "Empty info section"),
            RedisCommandError::EmptySetKeyOrValue => write!(f, "Empty set key or value"),
            RedisCommandError::EmptyReplConfCommand => write!(f, "Empty replconf command"),
            RedisCommandError::EmptyReplConfValue => write!(f, "Empty replconf value"),
        }
    }
}

impl ToRedisBytes for RedisCommandError {
    fn to_redis_bytes(&self) -> Vec<u8> {
        format!("${}\r\n{}\r\n", self.to_string().len(), self)
            .as_bytes()
            .to_vec()
    }
}
