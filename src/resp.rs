pub struct BulkString {
    bulks: Vec<Bulk>,
}
impl BulkString {
    pub fn bulks(&self) -> &Vec<Bulk> {
        &self.bulks
    }
    pub fn from_bytes(buf: &[u8]) -> BulkString {
        let mut message = std::str::from_utf8(buf).unwrap().split("\r\n");
        let length = message.next().unwrap().replace('*', "");
        let length = match length.parse::<usize>() {
            Ok(length) => length,
            Err(e) => {
                eprintln!("Can't parse {:?} as int", length);
                panic!("{e}");
            }
        };

        let mut bulks = vec![];
        for _ in 0..length {
            bulks.push(Bulk::build_from_iter(&mut message))
        }
        BulkString { bulks }
    }
}

pub enum RedisResponse {
    Null,
    Ok,
    Unimplemented,
    Pong,
}

impl ToRedisBytes for RedisResponse {
    fn to_redis_bytes(&self) -> Vec<u8> {
        match self {
            RedisResponse::Null => "$-1\r\n".as_bytes().to_vec(),
            RedisResponse::Ok => "+OK\r\n".as_bytes().to_vec(),
            RedisResponse::Unimplemented => unimplemented!(),
            RedisResponse::Pong => "$4\r\nPONG\r\n".as_bytes().to_vec(),
        }
    }
}

#[derive(Clone)]
pub struct Bulk {
    length: usize,
    data: String,
}
impl Bulk {
    pub fn length(&self) -> usize {
        self.length
    }
    pub fn data(&self) -> String {
        self.data.to_string()
    }
    fn build_from_iter(message: &mut std::str::Split<'_, &str>) -> Bulk {
        let length = message
            .next()
            .unwrap()
            .replace('$', "")
            .parse::<usize>()
            .unwrap();
        let data = message.next().unwrap().trim().to_string();
        Bulk { length, data }
    }
    pub fn from_bytes(buf: &[u8]) -> Bulk {
        let mut message = std::str::from_utf8(buf).unwrap().lines();
        let length = message
            .next()
            .unwrap()
            .replace('$', "")
            .parse::<usize>()
            .unwrap();
        let data = message.next().unwrap().trim().to_string();
        Bulk { length, data }
    }

    pub fn from_string(s: &str) -> Bulk {
        Bulk {
            length: s.len(),
            data: s.to_string(),
        }
    }
}

pub trait ToRedisBytes {
    fn to_redis_bytes(&self) -> Vec<u8>;
}
impl ToRedisBytes for Bulk {
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
        let bulk = Bulk::from_bytes(buf);
        assert_eq!(bulk.length(), 5);
        assert_eq!(bulk.data(), "hello");
    }

    #[test]
    fn test_bulk_to_redis_bytes() {
        let bulk = Bulk {
            length: 5,
            data: "hello".to_string(),
        };
        assert_eq!(bulk.to_redis_bytes(), b"$5\r\nhello\r\n");
    }

    #[test]
    fn test_bulk_string_from_bytes() {
        let buf = b"*2\r\n$3\r\nfoo\r\n$3\r\nbar\r\n";
        let bulk_string = BulkString::from_bytes(buf);
        assert_eq!(bulk_string.bulks().len(), 2);
        assert_eq!(bulk_string.bulks()[0].data(), "foo");
        assert_eq!(bulk_string.bulks()[1].data(), "bar");
    }

    #[test]
    fn test_redis_response_to_redis_bytes() {
        assert_eq!(RedisResponse::Null.to_redis_bytes(), b"$-1\r\n");
        assert_eq!(RedisResponse::Ok.to_redis_bytes(), b"+OK\r\n");
        assert_eq!(RedisResponse::Pong.to_redis_bytes(), b"$4\r\nPONG\r\n");
    }

    #[test]
    #[should_panic(expected = "not implemented")]
    fn test_redis_response_unimplemented() {
        RedisResponse::Unimplemented.to_redis_bytes();
    }

    #[test]
    fn test_string_to_redis_bytes() {
        let s = "hello".to_string();
        assert_eq!(s.to_redis_bytes(), b"$5\r\nhello\r\n");
    }
}
