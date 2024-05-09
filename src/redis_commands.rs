use std::time::Duration;

use crate::resp::{Array, BulkString, SimpleString, ToRedisBytes, Type};

pub enum RedisCommands {
    Ping,
    Echo(Vec<BulkString>),
    Get(String),
    Set((String, String, Option<Duration>)),
    Info(String),
    Replconf(String, String),
}
impl ToRedisBytes for RedisCommands {
    fn to_redis_bytes(&self) -> Vec<u8> {
        match self {
            Self::Ping => format!("*1\r\n${}\r\n{}\r\n", "PING".len(), "PING")
                .as_bytes()
                .to_vec(),
            Self::Echo(_) => todo!(),
            Self::Get(_) => todo!(),
            Self::Set(_) => todo!(),
            Self::Info(_) => todo!(),
            Self::Replconf(_, _) => todo!(),
        }
    }
}
impl RedisCommands {
    pub fn parse(command: &Type) -> Result<Self, RedisCommandError> {
        let redis_command = match command {
            Type::Array(array) => Self::handle_array(array)?,
            Type::BulkString(bulkstring) => Self::handle_bulkstring(bulkstring)?,
            Type::SimpleString(simplestring) => Self::handle_simplestring(simplestring)?,
        };
        Ok(redis_command)
    }

    fn handle_array(array: &Array) -> Result<Self, RedisCommandError> {
        let bulkstrings = array.bulkstrings();

        let command = bulkstrings.first().ok_or(RedisCommandError::EmptyCommand)?;
        match command.to_string().to_lowercase().as_str() {
            "ping" => Ok(Self::Ping),
            "echo" => Ok(Self::Echo(
                bulkstrings.get(1..).unwrap_or_default().to_vec(),
            )),
            "get" => Ok(Self::Get(
                bulkstrings
                    .get(1)
                    .ok_or(RedisCommandError::EmptyGetCommand)?
                    .to_string(),
            )),
            "set" => {
                let key = bulkstrings
                    .get(1)
                    .ok_or(RedisCommandError::EmptySetKeyOrValue)?
                    .to_string();
                let value = bulkstrings
                    .get(2)
                    .ok_or(RedisCommandError::EmptySetKeyOrValue)?
                    .to_string();
                let expiration = if bulkstrings.get(3).is_some() {
                    Some(Duration::from_millis(
                        bulkstrings
                            .get(4)
                            .ok_or(RedisCommandError::MissingSetExpiration)?
                            .to_string()
                            .parse::<u64>()
                            .map_err(|e| RedisCommandError::InvalidSetExpiration(e.to_string()))?,
                    ))
                } else {
                    None
                };
                Ok(Self::Set((key, value, expiration)))
            }
            "info" => Ok(Self::Info(
                bulkstrings
                    .get(1)
                    .ok_or(RedisCommandError::EmptyInfoSection)?
                    .to_string(),
            )),
            "replconf" => {
                let command = bulkstrings
                    .get(1)
                    .ok_or(RedisCommandError::EmptyReplConfCommand)?
                    .to_string();
                let value = bulkstrings
                    .get(2)
                    .ok_or(RedisCommandError::EmptyReplConfValue)?
                    .to_string();
                Ok(Self::Replconf(command, value))
            }
            command => Err(RedisCommandError::InvalidCommand(command.to_string())),
        }
    }
    fn handle_simplestring(simplestring: &SimpleString) -> Result<Self, RedisCommandError> {
        let command = simplestring.data().trim().to_lowercase();
        match command.as_str() {
            "ping" => Ok(Self::Ping),
            _ => Err(RedisCommandError::InvalidCommand(command)),
        }
    }
    fn handle_bulkstring(_bulkstring: &BulkString) -> Result<Self, RedisCommandError> {
        unimplemented!("Handle bulkstring in RedisCommand")
    }
}

#[derive(Debug)]
pub enum RedisCommandError {
    InvalidCommand(String),
    EmptyCommand,
    EmptyGetCommand,
    MissingSetExpiration,
    InvalidSetExpiration(String),
    EmptyInfoSection,
    EmptySetKeyOrValue,
    EmptyReplConfCommand,
    EmptyReplConfValue,
}
impl std::fmt::Display for RedisCommandError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidCommand(command) => write!(f, "Invalid command: '{command}'"),
            Self::EmptyCommand => write!(f, "Empty command"),
            Self::EmptyGetCommand => write!(f, "Empty get command"),
            Self::InvalidSetExpiration(expiration) => {
                write!(f, "Invalid set expiration: '{expiration}'")
            }
            Self::MissingSetExpiration => write!(f, "Expiration asked but no value given"),
            Self::EmptyInfoSection => write!(f, "Empty info section"),
            Self::EmptySetKeyOrValue => write!(f, "Empty set key or value"),
            Self::EmptyReplConfCommand => write!(f, "Empty replconf command"),
            Self::EmptyReplConfValue => write!(f, "Empty replconf value"),
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
