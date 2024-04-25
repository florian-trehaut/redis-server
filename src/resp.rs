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
#[derive(Clone)]
pub struct Bulk {
    length: usize,
    data: String,
}
impl Bulk {
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
    pub fn from_data(data: &str) -> Bulk {
        Bulk {
            length: data.len(),
            data: data.to_string(),
        }
    }
    pub fn to_bytes(&self) -> Vec<u8> {
        format!("${}\r\n{}\r\n", self.length, &self.data)
            .as_bytes()
            .to_vec()
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
