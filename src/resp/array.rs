use std::fmt::Display;

use super::{
    bulkstring::{BulkString, BulkStringError},
    ToRedisBytes,
};

#[derive(Clone, Debug)]
pub struct Array {
    bulkstrings: Vec<BulkString>,
}
impl Array {
    pub const fn bulkstrings(&self) -> &Vec<BulkString> {
        &self.bulkstrings
    }
    pub fn from_bytes(buf: &[u8]) -> Result<Self, ArrayError> {
        let mut message = std::str::from_utf8(buf)?
            .split("\r\n")
            .map(|s| s.trim().to_string());
        let length = message
            .next()
            .ok_or(ArrayError::MissingLength)?
            .replace('*', "");
        let length = length.parse::<usize>()?;

        let mut bulks = vec![];
        for _ in 0..length {
            bulks.push(BulkString::build_from_iter(&mut message)?);
        }
        Ok(Self { bulkstrings: bulks })
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

#[derive(Debug, Clone)]
pub enum ArrayError {
    Utf8Error(std::str::Utf8Error),
    ParseIntError(std::num::ParseIntError),
    MissingLength,
    MissingData,
}

impl From<std::str::Utf8Error> for ArrayError {
    fn from(err: std::str::Utf8Error) -> Self {
        Self::Utf8Error(err)
    }
}

impl From<std::num::ParseIntError> for ArrayError {
    fn from(err: std::num::ParseIntError) -> Self {
        Self::ParseIntError(err)
    }
}
impl From<BulkStringError> for ArrayError {
    fn from(err: BulkStringError) -> Self {
        match err {
            BulkStringError::Utf8Error(err) => Self::Utf8Error(err),
            BulkStringError::ParseIntError(err) => Self::ParseIntError(err),
            BulkStringError::MissingLength => Self::MissingLength,
            BulkStringError::MissingData => Self::MissingData,
        }
    }
}

impl Display for ArrayError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Utf8Error(err) => write!(f, "{err}"),
            Self::ParseIntError(err) => write!(f, "{err}"),
            Self::MissingLength => write!(f, "Missing length in RESP Array"),
            Self::MissingData => {
                write!(f, "Missing data in one of the bulks in RESP Array")
            }
        }
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
        let array = Array::from_bytes(b"*2\r\n$5\r\nhello\r\n$5\r\nworld\r\n").unwrap();
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

    #[test]
    fn test_array_error_display() {
        let error = ArrayError::MissingLength;
        assert_eq!(format!("{error}"), "Missing length in RESP Array");
    }
}
