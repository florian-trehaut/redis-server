use std::fmt::Display;

use super::ToRedisBytes;

pub enum RedisResponse {
    Null,
    Ok,
    Pong,
    _InvalidBulk,
}
const NULL_RESPONSE: &[u8] = b"$-1\r\n";
const OK_RESPONSE: &[u8] = b"+OK\r\n";
const PONG_RESPONSE: &[u8] = b"+PONG\r\n";
const INVALID_BULK_RESPONSE: &[u8] = b"$12\r\nInvalid bulk\r\n";

impl RedisResponse {
    pub fn from_bytes(buf: &[u8]) -> Self {
        match buf {
            NULL_RESPONSE => Self::Null,
            OK_RESPONSE => Self::Ok,
            PONG_RESPONSE => Self::Pong,
            INVALID_BULK_RESPONSE => Self::_InvalidBulk,
            _ => panic!("Invalid Redis response"),
        }
    }
}

impl Display for RedisResponse {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            std::str::from_utf8(&self.to_redis_bytes()).expect("Redis response is not valid UTF-8")
        )
    }
}

impl ToRedisBytes for RedisResponse {
    fn to_redis_bytes(&self) -> Vec<u8> {
        match self {
            Self::Null => NULL_RESPONSE.to_vec(),
            Self::Ok => OK_RESPONSE.to_vec(),
            Self::Pong => PONG_RESPONSE.to_vec(),
            Self::_InvalidBulk => INVALID_BULK_RESPONSE.to_vec(),
        }
    }
}
