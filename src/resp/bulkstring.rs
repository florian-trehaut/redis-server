use std::fmt::Display;

use super::ToRedisBytes;

#[derive(Clone, Debug, PartialEq, Eq)]
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
    pub fn build_from_iter(message: &mut impl Iterator<Item = String>) -> Self {
        let length = message
            .next()
            .expect("Bulkstring has no length")
            .replace('$', "")
            .parse::<usize>()
            .expect("Bulkstring has length but cannot be parsed to int");
        let data = message
            .next()
            .expect("Bulkstring has no data")
            .trim()
            .to_string();
        Self { length, data }
    }
    pub fn _from_bytes(buf: &[u8]) -> Self {
        let binding = String::from_utf8_lossy(buf);
        let mut message = binding.lines();
        let length = match message.next().expect("bulkstring bytes has no size") {
            "+PONG" => {
                return Self {
                    length: 4,
                    data: "PONG".to_string(),
                }
            }
            s => s
                .parse::<usize>()
                .expect("Cannot parse bulkstring length as int"),
        };
        let data = message
            .next()
            .expect("No data found in bulkstring")
            .trim()
            .to_string();
        Self { length, data }
    }
}
impl From<&str> for BulkString {
    fn from(s: &str) -> Self {
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

impl ToRedisBytes for BulkString {
    fn to_redis_bytes(&self) -> Vec<u8> {
        format!("${}\r\n{}\r\n", self.length(), &self.data())
            .as_bytes()
            .to_vec()
    }
}
