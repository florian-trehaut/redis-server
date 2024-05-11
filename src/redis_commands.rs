use std::time::Duration;

use crate::resp::{Array, BulkString, SimpleString, ToRedisBytes, Type};
use crate::server_config::{Offset, ReplicationId};

#[derive(Debug, PartialEq, Eq)]
pub enum RedisCommands {
    Ping,
    Echo(Vec<BulkString>),
    Get(String),
    Set((String, String, Option<Duration>)),
    Info(String),
    Replconf(String, String),
    Psync(ReplicationId, Offset),
}
impl ToRedisBytes for RedisCommands {
    fn to_redis_bytes(&self) -> Vec<u8> {
        match self {
            Self::Ping => format!("*1\r\n${}\r\n{}\r\n", "PING".len(), "PING")
                .as_bytes()
                .to_vec(),
            Self::Psync(replication_id, offset) => format!(
                "*3\r\n$5\r\nPSYNC\r\n${}\r\n{replication_id}\r\n${}\r\n{offset}\r\n",
                replication_id.len(),
                offset.len()
            )
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

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_ping_command() {
        let command = Type::SimpleString(SimpleString::from_bytes(b"+PING\r\n\r\n").unwrap());
        let result = RedisCommands::parse(&command).unwrap();
        assert_eq!(result, RedisCommands::Ping);
    }

    #[test]
    fn test_parse_echo_command() {
        let command = Type::Array(Array::from(vec![
            BulkString::from("ECHO"),
            BulkString::from("Hello"),
            BulkString::from("World"),
        ]));
        let result = RedisCommands::parse(&command).unwrap();
        assert_eq!(
            result,
            RedisCommands::Echo(vec![BulkString::from("Hello"), BulkString::from("World"),])
        );
    }

    #[test]
    fn test_parse_get_command() {
        let command = Type::Array(Array::from(vec![
            BulkString::from("GET"),
            BulkString::from("mykey"),
        ]));
        let result = RedisCommands::parse(&command).unwrap();
        assert_eq!(result, RedisCommands::Get("mykey".to_string()));
    }

    #[test]
    fn test_parse_set_command() {
        let command = Type::Array(Array::from(vec![
            BulkString::from("SET"),
            BulkString::from("mykey"),
            BulkString::from("myvalue"),
            BulkString::from("EX"),
            BulkString::from("1000"),
        ]));
        let result = RedisCommands::parse(&command).unwrap();
        assert_eq!(
            result,
            RedisCommands::Set((
                "mykey".to_string(),
                "myvalue".to_string(),
                Some(Duration::from_millis(1000))
            ))
        );
    }

    #[test]
    fn test_parse_info_command() {
        let command = Type::Array(Array::from(vec![
            BulkString::from("INFO"),
            BulkString::from("server"),
        ]));
        let result = RedisCommands::parse(&command).unwrap();
        assert_eq!(result, RedisCommands::Info("server".to_string()));
    }

    #[test]
    fn test_parse_replconf_command() {
        let command = Type::Array(Array::from(vec![
            BulkString::from("REPLCONF"),
            BulkString::from("listening-port"),
            BulkString::from("1234"),
        ]));
        let result = RedisCommands::parse(&command).unwrap();
        assert_eq!(
            result,
            RedisCommands::Replconf("listening-port".to_string(), "1234".to_string())
        );
    }
}
