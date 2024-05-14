use crate::resp::{Array, BulkString, SimpleString, ToRedisBytes, Type};
use crate::server_config::{Offset, ReplicationId};
use std::fmt::Display;
use std::time::Duration;

#[derive(Debug, PartialEq, Eq)]
pub enum RedisCommands {
    Ping,
    Echo(Vec<BulkString>),
    Get(String),
    Set((String, String, Option<Duration>)),
    Info(String),
    Replconf(String, String),
    Psync(ReplicationId, Offset),
    FullResync(ReplicationId, Offset),
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
            Self::FullResync(replication_id, offset) => {
                format!("+FULLRESYNC {replication_id} {offset}\r\n")
                    .as_bytes()
                    .to_vec()
            }
            Self::Echo(_) => todo!(),
            Self::Get(_) => todo!(),
            Self::Set(_) => todo!(),
            Self::Info(_) => todo!(),
            Self::Replconf(_, _) => todo!(),
        }
    }
}
impl RedisCommands {
    pub fn parse(command: &Type) -> Self {
        match command {
            Type::Array(array) => Self::handle_array(array),
            Type::BulkString(bulkstring) => Self::handle_bulkstring(bulkstring),
            Type::SimpleString(simplestring) => Self::handle_simplestring(simplestring),
        }
    }

    fn handle_array(array: &Array) -> Self {
        let bulkstrings = array.bulkstrings();

        let command = bulkstrings.first().expect("No command found");
        match command.to_string().to_lowercase().as_str() {
            "ping" => Self::Ping,
            "echo" => Self::Echo(bulkstrings.get(1..).unwrap_or_default().to_vec()),
            "get" => Self::Get(bulkstrings.get(1).expect("No key found").to_string()),
            "set" => {
                let key = bulkstrings.get(1).expect("Set has no key").to_string();
                let value = bulkstrings.get(2).expect("Set has no value").to_string();
                let expiration = if bulkstrings.get(3).is_some() {
                    Some(Duration::from_millis(
                        bulkstrings
                            .get(4)
                            .expect("Set has expiration parameter but no expiration value")
                            .to_string()
                            .parse::<u64>()
                            .expect("Set has expiration parameter and value but value cannot be parsed as signed integer"),
                    ))
                } else {
                    None
                };
                Self::Set((key, value, expiration))
            }
            "info" => Self::Info(
                bulkstrings
                    .get(1)
                    .expect("Info requested but no category given")
                    .to_string(),
            ),
            "replconf" => {
                let command = bulkstrings
                    .get(1)
                    .expect("Replfconf has no command")
                    .to_string();
                let value = bulkstrings
                    .get(2)
                    .expect("Replconf has no value")
                    .to_string();
                Self::Replconf(command, value)
            }
            "psync" => {
                let replication_id = ReplicationId::parse(Some(
                    bulkstrings
                        .get(1)
                        .expect("Psync has no replication ID")
                        .to_string(),
                ));
                let replication_offset = Offset::parse(Some(
                    bulkstrings
                        .get(2)
                        .expect("Psync command has no replication offset")
                        .to_string()
                        .parse::<i8>()
                        .expect("Psync has replication offset but cannot be parsed as integer"),
                ));
                Self::Psync(replication_id, replication_offset)
            }
            command => unimplemented!("Command '{command}' cannot be parsed"),
        }
    }
    fn handle_simplestring(simplestring: &SimpleString) -> Self {
        let command = simplestring.data().trim().to_lowercase();
        match command.as_str() {
            "ping" => Self::Ping,
            _ => unimplemented!("Simplestring command is unimplemented"),
        }
    }
    fn handle_bulkstring(_bulkstring: &BulkString) -> Self {
        unimplemented!("Handle bulkstring in RedisCommand")
    }
}
impl Display for RedisCommands {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Ping => write!(f, "Ping"),
            Self::Echo(_) => write!(f, "Echo"),
            Self::Get(_) => write!(f, "Get"),
            Self::Set(_) => write!(f, "Set"),
            Self::Info(_) => write!(f, "Info"),
            Self::Replconf(_, _) => write!(f, "Replconf"),
            Self::Psync(_, _) => write!(f, "Psync"),
            Self::FullResync(_, _) => write!(f, "FullResync"),
        }
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_ping_command() {
        let command = Type::SimpleString(SimpleString::from_bytes(b"+PING\r\n\r\n"));
        let result = RedisCommands::parse(&command);
        assert_eq!(result, RedisCommands::Ping);
    }

    #[test]
    fn test_parse_echo_command() {
        let command = Type::Array(Array::from(vec![
            BulkString::from("ECHO"),
            BulkString::from("Hello"),
            BulkString::from("World"),
        ]));
        let result = RedisCommands::parse(&command);
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
        let result = RedisCommands::parse(&command);
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
        let result = RedisCommands::parse(&command);
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
        let result = RedisCommands::parse(&command);
        assert_eq!(result, RedisCommands::Info("server".to_string()));
    }

    #[test]
    fn test_parse_replconf_command() {
        let command = Type::Array(Array::from(vec![
            BulkString::from("REPLCONF"),
            BulkString::from("listening-port"),
            BulkString::from("1234"),
        ]));
        let result = RedisCommands::parse(&command);
        assert_eq!(
            result,
            RedisCommands::Replconf("listening-port".to_string(), "1234".to_string())
        );
    }
}
