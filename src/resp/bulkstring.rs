use std::fmt::Display;

use super::ToRedisBytes;

#[derive(Clone, Debug)]
pub struct BulkString {
    length: usize,
    data: String,
}
impl BulkString {
    pub const fn length(&self) -> usize {
        self.length
    }
    pub fn data(&self) -> String {
        self.data.to_string()
    }
    pub fn build_from_iter(
        message: &mut impl Iterator<Item = String>,
    ) -> Result<Self, BulkStringError> {
        let length = message
            .next()
            .ok_or(BulkStringError::MissingLength)?
            .replace('$', "")
            .parse::<usize>()?;
        let data = message
            .next()
            .ok_or(BulkStringError::MissingData)?
            .trim()
            .to_string();
        Ok(Self { length, data })
    }
    pub fn _from_bytes(buf: &[u8]) -> Result<Self, BulkStringError> {
        let mut message = std::str::from_utf8(buf)?.lines();
        let length = match message.next().ok_or(BulkStringError::MissingLength)? {
            "+PONG" => {
                return Ok(Self {
                    length: 4,
                    data: "PONG".to_string(),
                })
            }
            s => s.parse::<usize>()?,
        };
        let data = message
            .next()
            .ok_or(BulkStringError::MissingData)?
            .trim()
            .to_string();
        Ok(Self { length, data })
    }

    pub fn from_string(s: &str) -> Self {
        Self {
            length: s.len(),
            data: s.to_string(),
        }
    }
}
impl Display for BulkString {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.data)
    }
}

#[derive(Debug, Clone)]
pub enum BulkStringError {
    Utf8Error(std::str::Utf8Error),
    ParseIntError(std::num::ParseIntError),
    MissingLength,
    MissingData,
}
impl From<std::str::Utf8Error> for BulkStringError {
    fn from(err: std::str::Utf8Error) -> Self {
        Self::Utf8Error(err)
    }
}
impl From<std::num::ParseIntError> for BulkStringError {
    fn from(err: std::num::ParseIntError) -> Self {
        Self::ParseIntError(err)
    }
}
impl Display for BulkStringError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Utf8Error(err) => write!(f, "{err}"),
            Self::ParseIntError(err) => write!(f, "{err}"),
            Self::MissingLength => write!(f, "Missing length in bulk string"),
            Self::MissingData => write!(f, "Missing data in bulk string"),
        }
    }
}
impl ToRedisBytes for BulkString {
    fn to_redis_bytes(&self) -> Vec<u8> {
        format!("${}\r\n{}\r\n", self.length(), &self.data())
            .as_bytes()
            .to_vec()
    }
}
