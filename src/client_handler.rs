use std::{
    cmp::Ordering,
    fmt::Display,
    io::{Read, Write},
    net::TcpStream,
    time::{Duration, Instant},
};

use crate::{
    redis_commands::RedisCommands,
    redis_info::RedisInfo,
    resp::{BulkString, RedisResponse, RespArray, ToRedisBytes},
    RedisStore, RedisValue, ServerConfig,
};

pub struct ClientHandler {
    store: RedisStore,
    server_info: RedisInfo,
}

impl ClientHandler {
    pub fn new(store: RedisStore, server_config: ServerConfig) -> Self {
        let server_info = RedisInfo::new(server_config);
        Self { store, server_info }
    }

    pub fn handle(&mut self, stream: &mut TcpStream) {
        let mut buf = [0; 512];
        while let Ok(n) = stream.read(&mut buf) {
            if n == 0 {
                break;
            }
            let command = match RespArray::from_bytes(&buf[..n]) {
                Ok(command) => command,
                Err(e) => {
                    eprintln!("Error parsing command: {:?}", e);
                    self.respond(RedisResponse::Null, stream);
                    continue;
                }
            };
            println!("Received RESPArray: {command}");
            let redis_command = match RedisCommands::parse(command) {
                Ok(redis_command) => redis_command,
                Err(e) => {
                    eprintln!("Error parsing command: {:?}", e);
                    self.respond(e, stream);
                    continue;
                }
            };
            match redis_command {
                RedisCommands::Ping => self.ping(stream),
                RedisCommands::Echo(message) => self.echo(&message, stream),
                RedisCommands::Get(key) => self.get(key, stream),
                RedisCommands::Set((key, value, expiration)) => {
                    self.set(key, value, expiration, stream)
                }
                RedisCommands::Info(section) => self.info(section, stream),
                RedisCommands::Replconf(command, value) => self.replconf(command, value, stream),
            }
        }
    }

    fn ping(&self, stream: &mut TcpStream) {
        self.respond(RedisResponse::Pong, stream);
    }

    fn echo(&self, message: &[BulkString], stream: &mut TcpStream) {
        let message = message
            .iter()
            .map(|bulk| bulk.data())
            .collect::<Vec<String>>()
            .join("");
        let message = BulkString::from_string(&message);
        self.respond(message, stream);
    }
    fn set(
        &self,
        key: String,
        value: String,
        expiration: Option<Duration>,
        stream: &mut TcpStream,
    ) {
        let value = RedisValue::new(value, expiration);

        let mut store = match self.store.lock() {
            Ok(store) => store,
            Err(e) => {
                eprintln!("Error locking store: {}", e);
                self.respond(ClientHandlerError::PoisonedStore, stream);
                return;
            }
        };
        match store.insert(key.clone(), value.clone()) {
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
        self.respond(RedisResponse::Ok, stream)
    }

    fn get(&self, key: String, stream: &mut TcpStream) {
        let store = match self.store.lock() {
            Ok(store) => store,
            Err(e) => {
                eprintln!("Error locking store: {}", e);
                self.respond(ClientHandlerError::PoisonedStore, stream);
                return;
            }
        };
        let redis_value = match store.get(&key) {
            Some(redis_value) => redis_value,
            None => {
                println!("Get -- Key:{key} has not been found");
                self.respond(RedisResponse::Null, stream);
                return;
            }
        };
        let expiration = match redis_value.expiration() {
            Some(expiration) => expiration,
            None => {
                println!("Get -- Key:{key} has been found and have no expiration");
                self.respond(redis_value.value(), stream);
                return;
            }
        };

        match Instant::now().cmp(&expiration) {
            Ordering::Equal | Ordering::Less => {
                println!("Get -- Key:{key} has been found and is not expired");
                self.respond(redis_value.value(), stream)
            }
            Ordering::Greater => {
                println!("Get -- Key:{key} has been found but is expired");
                self.respond(RedisResponse::Null, stream)
            }
        }
    }

    fn info(&self, section: String, stream: &mut TcpStream) {
        let info = match section.to_lowercase().as_str() {
            "replication" => self.server_info.to_bulk_string(),
            _ => BulkString::from_string("Unknown section"),
        };
        self.respond(info, stream);
    }

    fn replconf(&self, _command: String, _value: String, stream: &mut TcpStream) {
        self.respond(RedisResponse::Ok, stream)
    }

    fn respond(&self, response: impl ToRedisBytes, stream: &mut TcpStream) {
        println!(
            "Responding with: {:?}",
            std::str::from_utf8(&response.to_redis_bytes())
        );
        let response = response.to_redis_bytes();
        match stream.write_all(&response) {
            Ok(_) => (),
            Err(e) => eprintln!("Error writing to stream: {:?}", e),
        }
    }
}

enum ClientHandlerError {
    PoisonedStore,
}
impl Display for ClientHandlerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ClientHandlerError::PoisonedStore => write!(f, "Poisoned store"),
        }
    }
}
impl ToRedisBytes for ClientHandlerError {
    fn to_redis_bytes(&self) -> Vec<u8> {
        format!("${}\r\n{}\r\n", self.to_string().len(), self)
            .as_bytes()
            .to_vec()
    }
}
