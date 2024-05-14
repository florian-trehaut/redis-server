pub use self::{array::Array, bulkstring::BulkString, simple_string::SimpleString};

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
    pub fn from_bytes(buf: &[u8]) -> Self {
        match buf[0] {
            b'*' => Self::Array(Array::from_bytes(buf)),
            b'$' => Self::BulkString(BulkString::_from_bytes(buf)),
            b'+' => Self::SimpleString(SimpleString::from_bytes(buf)),
            _ => panic!(
                "Cannot define command type of '{}'",
                String::from_utf8_lossy(buf)
            ),
        }
    }
}
