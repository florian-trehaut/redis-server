use std::fmt::Display;

use super::{bulkstring::BulkString, ToRedisBytes};

#[derive(Clone, Debug)]
pub struct Array {
    bulkstrings: Vec<BulkString>,
}
impl Array {
    pub const fn bulkstrings(&self) -> &Vec<BulkString> {
        &self.bulkstrings
    }
    pub fn from_bytes(buf: &[u8]) -> Self {
        let binding = String::from_utf8_lossy(buf);
        let mut message = binding.split("\r\n").map(|s| s.trim().to_string());
        let length = message
            .next()
            .expect("Resp array is missing length")
            .replace('*', "");
        let length = length
            .parse::<usize>()
            .expect("Resp array has length but is unparsable to int");

        let mut bulks = vec![];
        for _ in 0..length {
            bulks.push(BulkString::build_from_iter(&mut message));
        }
        Self { bulkstrings: bulks }
    }
    pub fn from_string(s: &str) -> Self {
        let bulkstrings: Vec<BulkString> = s.split_whitespace().map(BulkString::from).collect();
        Self { bulkstrings }
    }
}
impl From<Vec<BulkString>> for Array {
    fn from(bulkstrings: Vec<BulkString>) -> Self {
        Self { bulkstrings }
    }
}
impl Display for Array {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut result = String::new();
        result.push_str(&format!("*{}\r\n", self.bulkstrings.len()));
        for bulk in &self.bulkstrings {
            result.push_str(
                &bulk
                    .to_redis_bytes()
                    .iter()
                    .map(|&c| c as char)
                    .collect::<String>(),
            );
        }
        write!(f, "{result}")
    }
}
impl ToRedisBytes for Array {
    fn to_redis_bytes(&self) -> Vec<u8> {
        format!("{self}").into_bytes()
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn test_array_from_string() {
        let array = Array::from_string("hello world");
        assert_eq!(array.bulkstrings().len(), 2);
        assert_eq!(array.bulkstrings()[0].to_string(), "hello");
        assert_eq!(array.bulkstrings()[1].to_string(), "world");
    }

    #[test]
    fn test_array_from_bytes() {
        let array = Array::from_bytes(b"*2\r\n$5\r\nhello\r\n$5\r\nworld\r\n");
        assert_eq!(array.bulkstrings().len(), 2);
        assert_eq!(array.bulkstrings()[0].to_string(), "hello");
        assert_eq!(array.bulkstrings()[1].to_string(), "world");
    }

    #[test]
    fn test_array_to_redis_bytes() {
        let array = Array::from_string("hello world");
        assert_eq!(
            array.to_redis_bytes(),
            b"*2\r\n$5\r\nhello\r\n$5\r\nworld\r\n".to_vec()
        );
    }

    #[test]
    fn test_array_display() {
        let array = Array::from_string("hello world");
        assert_eq!(format!("{array}"), "*2\r\n$5\r\nhello\r\n$5\r\nworld\r\n");
    }
}
