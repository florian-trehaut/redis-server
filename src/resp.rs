use std::fmt::Display;

#[derive(Clone, Debug)]
pub struct RespArray {
    bulks: Vec<BulkString>,
}
impl RespArray {
    pub fn bulks(&self) -> &Vec<BulkString> {
        &self.bulks
    }
    pub fn from_bytes(buf: &[u8]) -> Result<RespArray, RespArrayError> {
        let mut message = std::str::from_utf8(buf)?.split("\r\n");
        let length = message
            .next()
            .ok_or(RespArrayError::MissingLength)?
            .replace('*', "");
        let length = length.parse::<usize>()?;

        let mut bulks = vec![];
        for _ in 0..length {
            bulks.push(BulkString::build_from_iter(&mut message)?)
        }
        Ok(RespArray { bulks })
    }
}
impl Display for RespArray {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut result = String::new();
        result.push_str(&format!("*{}\r\n", self.bulks.len()));
        for bulk in &self.bulks {
            result.push_str(
                &bulk
                    .to_redis_bytes()
                    .iter()
                    .map(|&c| c as char)
                    .collect::<String>(),
            );
        }
        write!(f, "{}", result)
    }
}

#[derive(Debug, Clone)]
pub enum RespArrayError {
    Utf8Error(std::str::Utf8Error),
    ParseIntError(std::num::ParseIntError),
    MissingLength,
    MissingData,
}

impl From<std::str::Utf8Error> for RespArrayError {
    fn from(err: std::str::Utf8Error) -> Self {
        Self::Utf8Error(err)
    }
}

impl From<std::num::ParseIntError> for RespArrayError {
    fn from(err: std::num::ParseIntError) -> Self {
        Self::ParseIntError(err)
    }
}
impl From<BulkStringError> for RespArrayError {
    fn from(err: BulkStringError) -> Self {
        match err {
            BulkStringError::Utf8Error(err) => RespArrayError::Utf8Error(err),
            BulkStringError::ParseIntError(err) => RespArrayError::ParseIntError(err),
            BulkStringError::MissingLength => RespArrayError::MissingLength,
            BulkStringError::MissingData => RespArrayError::MissingData,
        }
    }
}

impl Display for RespArrayError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RespArrayError::Utf8Error(err) => write!(f, "{}", err),
            RespArrayError::ParseIntError(err) => write!(f, "{}", err),
            RespArrayError::MissingLength => write!(f, "Missing length in RESP Array"),
            RespArrayError::MissingData => {
                write!(f, "Missing data in one of the bulks in RESP Array")
            }
        }
    }
}

pub enum RedisResponse {
    Null,
    Ok,
    Pong,
    _InvalidBulk,
}

impl ToRedisBytes for RedisResponse {
    fn to_redis_bytes(&self) -> Vec<u8> {
        match self {
            RedisResponse::Null => "$-1\r\n".as_bytes().to_vec(),
            RedisResponse::Ok => "+OK\r\n".as_bytes().to_vec(),
            RedisResponse::Pong => "+PONG\r\n".as_bytes().to_vec(),
            RedisResponse::_InvalidBulk => "$12\r\nInvalid bulk\r\n".as_bytes().to_vec(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct BulkString {
    length: usize,
    data: String,
}
impl BulkString {
    pub fn length(&self) -> usize {
        self.length
    }
    pub fn data(&self) -> String {
        self.data.to_string()
    }
    fn build_from_iter(
        message: &mut std::str::Split<'_, &str>,
    ) -> Result<BulkString, BulkStringError> {
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
        Ok(BulkString { length, data })
    }
    pub fn from_bytes(buf: &[u8]) -> Result<BulkString, BulkStringError> {
        let mut message = std::str::from_utf8(buf)?.lines();
        let length = match message.next().ok_or(BulkStringError::MissingLength)? {
            "+PONG" => {
                return Ok(BulkString {
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
        Ok(BulkString { length, data })
    }

    pub fn from_string(s: &str) -> BulkString {
        BulkString {
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
            BulkStringError::Utf8Error(err) => write!(f, "{}", err),
            BulkStringError::ParseIntError(err) => write!(f, "{}", err),
            BulkStringError::MissingLength => write!(f, "Missing length in bulk string"),
            BulkStringError::MissingData => write!(f, "Missing data in bulk string"),
        }
    }
}

pub trait ToRedisBytes {
    fn to_redis_bytes(&self) -> Vec<u8>;
}
impl ToRedisBytes for BulkString {
    fn to_redis_bytes(&self) -> Vec<u8> {
        format!("${}\r\n{}\r\n", self.length(), &self.data())
            .as_bytes()
            .to_vec()
    }
}
impl ToRedisBytes for String {
    fn to_redis_bytes(&self) -> Vec<u8> {
        format!("${}\r\n{}\r\n", self.len(), self)
            .as_bytes()
            .to_vec()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bulk_from_bytes() {
        let buf = b"$5\r\nhello\r\n";
        let bulk = BulkString::from_bytes(buf);
        assert_eq!(bulk.clone().unwrap().length(), 5);
        assert_eq!(bulk.clone().unwrap().data(), "hello");
    }

    #[test]
    fn test_bulk_to_redis_bytes() {
        let bulk = BulkString {
            length: 5,
            data: "hello".to_string(),
        };
        assert_eq!(bulk.to_redis_bytes(), b"$5\r\nhello\r\n");
    }

    #[test]
    fn test_bulk_string_from_bytes() {
        let buf = b"*2\r\n$3\r\nfoo\r\n$3\r\nbar\r\n";
        let bulk_string = RespArray::from_bytes(buf);
        assert_eq!(bulk_string.clone().unwrap().bulks().len(), 2);
        assert_eq!(bulk_string.clone().unwrap().bulks()[0].data(), "foo");
        assert_eq!(bulk_string.clone().unwrap().bulks()[1].data(), "bar");
    }

    #[test]
    fn test_redis_response_to_redis_bytes() {
        assert_eq!(RedisResponse::Null.to_redis_bytes(), b"$-1\r\n");
        assert_eq!(RedisResponse::Ok.to_redis_bytes(), b"+OK\r\n");
        assert_eq!(RedisResponse::Pong.to_redis_bytes(), b"$4\r\nPONG\r\n");
    }

    #[test]
    fn test_string_to_redis_bytes() {
        let s = "hello".to_string();
        assert_eq!(s.to_redis_bytes(), b"$5\r\nhello\r\n");
    }
    #[test]
    fn test_bulk_string_from_bytes_error() {
        let buf = b"*2\r\n$3\r\nfoo\r\n";
        let result = RespArray::from_bytes(buf);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            RespArrayError::ParseIntError(_)
        ));
    }

    #[test]
    fn test_bulk_string_from_bytes_utf8_error() {
        let buf = [0, 159, 146, 150]; // invalid UTF-8 sequence
        let result = RespArray::from_bytes(&buf);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), RespArrayError::Utf8Error(_)));
    }

    #[test]
    fn test_bulk_string_from_bytes_parse_int_error() {
        let buf = b"*not_a_number\r\n$3\r\nfoo\r\n$3\r\nbar\r\n";
        let result = RespArray::from_bytes(buf);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            RespArrayError::ParseIntError(_)
        ));
    }
}
