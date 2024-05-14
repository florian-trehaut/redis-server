use std::{
    cmp::Ordering,
    fmt::Display,
    io::{Read, Write},
    net::TcpStream,
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};

use crate::{
    redis_commands::RedisCommands,
    redis_info::RedisInfo,
    resp::{redis_response::RedisResponse, BulkString, ToRedisBytes, Type},
    RedisStore, RedisValue,
};

pub trait CommonCommands {
    fn ping(stream: &mut TcpStream) {
        Self::respond(&RedisResponse::Pong, stream);
    }

    fn echo(message: &[BulkString], stream: &mut TcpStream) {
        let message = message.iter().map(BulkString::data).collect::<String>();
        let message = BulkString::from(message.as_str());
        Self::respond(&message, stream);
    }
    fn set(
        store: &RedisStore,
        key: &str,
        value: String,
        expiration: Option<Duration>,
        stream: &mut TcpStream,
    ) {
        let value = RedisValue::new(value, expiration);
        println!("Inserting key:{key} with value:{value}");

        match store.lock() {
            Ok(mut store) => store.insert(key.to_string(), value),
            Err(e) => {
                eprintln!("Error locking store: {e}");
                Self::respond(&ClientHandlerError::PoisonedStore, stream);
                return;
            }
        };
        Self::respond(&RedisResponse::Ok, stream);
    }

    fn get(store: &RedisStore, key: &str, stream: &mut TcpStream) {
        let redis_value = match store.lock() {
            Ok(store) => store.get(key).cloned(),
            Err(e) => {
                eprintln!("Error locking store: {e}");
                Self::respond(&ClientHandlerError::PoisonedStore, stream);
                return;
            }
        };
        let Some(redis_value) = redis_value else {
            println!("Get -- Key:{key} has not been found");
            Self::respond(&RedisResponse::Null, stream);
            return;
        };
        let Some(expiration) = redis_value.expiration() else {
            println!("Get -- Key:{key} has been found and have no expiration");
            Self::respond(&redis_value.value(), stream);
            return;
        };

        match Instant::now().cmp(&expiration) {
            Ordering::Equal | Ordering::Less => {
                println!("Get -- Key:{key} has been found and is not expired");
                Self::respond(&redis_value.value(), stream);
            }
            Ordering::Greater => {
                println!("Get -- Key:{key} has been found but is expired");
                Self::respond(&RedisResponse::Null, stream);
            }
        }
    }

    fn info(server_info: &Arc<Mutex<RedisInfo>>, section: &str, stream: &mut TcpStream) {
        let info = match section.to_lowercase().as_str() {
            "replication" => server_info
                .lock()
                .expect("Poisonned lock when getting server info")
                .to_bulk_string(),
            _ => BulkString::from("Unknown section"),
        };
        Self::respond(&info, stream);
    }

    fn respond(response: &impl ToRedisBytes, stream: &mut TcpStream) {
        println!(
            "Responding with: {:?}",
            std::str::from_utf8(&response.to_redis_bytes())
        );
        let response = response.to_redis_bytes();
        match stream.write_all(&response) {
            Ok(()) => (),
            Err(e) => eprintln!("Error writing to stream: {e}"),
        }
    }

    fn parse_redis_command_from_stream(buf: &[u8; 512], n: usize) -> Option<RedisCommands> {
        let command = Type::from_bytes(&buf[..n]);
        let redis_command = RedisCommands::parse(&command);
        Some(redis_command)
    }

    fn match_redis_command(
        redis_command: RedisCommands,
        stream: &mut TcpStream,
        store: &Arc<Mutex<std::collections::HashMap<String, RedisValue>>>,
        redis_info: &Arc<Mutex<RedisInfo>>,
    );
}

pub trait ClientHandler: CommonCommands {
    fn handle(redis_info: Arc<Mutex<RedisInfo>>, store: RedisStore, stream: &mut TcpStream) {
        let mut buf = [0; 512];
        while let Ok(n) = stream.read(&mut buf) {
            if n == 0 {
                break;
            }
            println!(
                "Received command: '{}'",
                String::from_utf8_lossy(&buf[0..n])
            );
            let Some(redis_command) = Self::parse_redis_command_from_stream(&buf, n) else {
                eprintln!(
                    "Cannot parse command {}",
                    String::from_utf8_lossy(&buf[0..n])
                );
                continue;
            };
            Self::match_redis_command(redis_command, stream, &store, &redis_info);
        }
    }
}

enum ClientHandlerError {
    PoisonedStore,
}
impl Display for ClientHandlerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::PoisonedStore => write!(f, "Poisoned store"),
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
