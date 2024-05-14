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
    resp::{BulkString, RedisResponse, ToRedisBytes, Type},
    RedisStore, RedisValue,
};

pub trait ClientHandler {
    fn handle(redis_info: Arc<Mutex<RedisInfo>>, store: RedisStore, stream: &mut TcpStream) {
        let mut buf = [0; 512];
        while let Ok(n) = stream.read(&mut buf) {
            if n == 0 {
                break;
            }
            let Some(redis_command) = Self::parse_redis_command_from_stream(buf, n, stream) else {
                continue;
            };
            match redis_command {
                RedisCommands::Ping => Self::ping(stream),
                RedisCommands::Echo(message) => Self::echo(&message, stream),
                RedisCommands::Get(key) => Self::get(&store, &key, stream),
                RedisCommands::Set((key, value, expiration)) => {
                    Self::set(&store, &key, value, expiration, stream);
                }
                RedisCommands::Info(section) => Self::info(&redis_info, &section, stream),
                RedisCommands::Replconf(_, _) => {
                    Self::respond(&RedisResponse::Ok, stream);
                }
                RedisCommands::Psync(_, _) => Self::psync(&redis_info, stream),
                RedisCommands::FullResync(_, _) => (),
            }
        }
    }

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

    fn psync(server_info: &Arc<Mutex<RedisInfo>>, stream: &mut TcpStream) {
        let command = RedisCommands::FullResync(
            server_info
                .lock()
                .expect("Poisonned store when getting server info")
                .master_replid()
                .to_owned(),
            server_info
                .lock()
                .expect("Poisonned store when getting server info")
                .master_repl_offset()
                .to_owned(),
        );
        Self::respond(&command, stream);
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

    fn parse_redis_command_from_stream(
        buf: [u8; 512],
        n: usize,
        stream: &mut TcpStream,
    ) -> Option<RedisCommands> {
        let command = match Type::from_bytes(&buf[..n]) {
            Ok(command) => command,
            Err(e) => {
                eprintln!("Error determining command type: {e}");
                Self::respond(&RedisResponse::Null, stream);
                return None;
            }
        };
        let redis_command = match RedisCommands::parse(&command) {
            Ok(redis_command) => redis_command,
            Err(e) => {
                eprintln!("Error parsing command: {e}");
                Self::respond(&RedisResponse::Null, stream);
                return None;
            }
        };
        Some(redis_command)
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
