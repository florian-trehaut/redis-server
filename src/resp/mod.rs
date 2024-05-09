use std::fmt::Display;

pub use self::{
    array::{Array, ArrayError},
    bulkstring::{BulkString, BulkStringError},
    redis_response::RedisResponse,
    simple_string::{SimpleString, SimpleStringError},
};

pub mod array;
pub mod bulkstring;
pub mod redis_response;
pub mod simple_string;

pub trait ToRedisBytes {
    fn to_redis_bytes(&self) -> Vec<u8>;
}
impl ToRedisBytes for String {
    fn to_redis_bytes(&self) -> Vec<u8> {
        format!("${}\r\n{}\r\n", self.len(), self)
            .as_bytes()
            .to_vec()
    }
}

pub enum Type {
    Array(Array),
    BulkString(BulkString),
    SimpleString(SimpleString),
}

impl Type {
    pub fn from_bytes(buf: &[u8]) -> Result<Self, TypeError> {
        match buf[0] {
            b'*' => Ok(Self::Array(Array::from_bytes(buf)?)),
            b'$' => Ok(Self::BulkString(BulkString::_from_bytes(buf)?)),
            b'+' => Ok(Self::SimpleString(SimpleString::from_bytes(buf)?)),
            _ => Err(TypeError::InvalidType),
        }
    }
}
pub enum TypeError {
    InvalidType,
    ArrayParseError(ArrayError),
    BulkStringParseError(BulkStringError),
    SimpleStringParseError(SimpleStringError),
}
impl From<ArrayError> for TypeError {
    fn from(err: ArrayError) -> Self {
        Self::ArrayParseError(err)
    }
}
impl From<BulkStringError> for TypeError {
    fn from(err: BulkStringError) -> Self {
        Self::BulkStringParseError(err)
    }
}
impl From<SimpleStringError> for TypeError {
    fn from(err: SimpleStringError) -> Self {
        Self::SimpleStringParseError(err)
    }
}
impl Display for TypeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidType => write!(f, "Invalid RESP type"),
            Self::ArrayParseError(err) => write!(f, "{err}"),
            Self::BulkStringParseError(err) => write!(f, "{err}"),
            Self::SimpleStringParseError(err) => write!(f, "{err}"),
        }
    }
}
