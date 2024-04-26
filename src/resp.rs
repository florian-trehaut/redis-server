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
