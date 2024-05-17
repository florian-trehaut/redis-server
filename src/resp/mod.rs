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

pub struct RDBFile {
    hex_content: String,
}
impl RDBFile {
    pub fn empty_file() -> Self {
        Self{hex_content: "524544495330303131fa0972656469732d76657205372e322e30fa0a72656469732d62697473c040fa056374696d65c26d08bc65fa08757365642d6d656dc2b0c41000fa08616f662d62617365c000fff06e3bfec0ff5aa2".to_string()}
    }
    pub fn length(&self) -> usize {
        self.hex_content.len() / 2
    }
}
impl ToRedisBytes for RDBFile {
    fn to_redis_bytes(&self) -> Vec<u8> {
        let mut buffer = vec![];
        buffer.extend_from_slice(b"$");
        buffer.extend_from_slice(self.length().to_string().as_bytes());
        buffer.extend_from_slice(b"\r\n");

        let content: Vec<u8> = (0..self.hex_content.len())
            .step_by(2)
            .map(|i| {
                u8::from_str_radix(&self.hex_content[i..i + 2], 16)
                    .expect("Conversion hexadécimale invalide")
            })
            .collect();

        buffer.extend_from_slice(&content);
        buffer
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_file() {
        let rdb_file = RDBFile::empty_file();
        assert_eq!(rdb_file.hex_content, "524544495330303131fa0972656469732d76657205372e322e30fa0a72656469732d62697473c040fa056374696d65c26d08bc65fa08757365642d6d656dc2b0c41000fa08616f662d62617365c000fff06e3bfec0ff5aa2");
    }

    #[test]
    fn test_length() {
        let rdb_file = RDBFile::empty_file();
        // La longueur attendue est la moitié de la longueur de la chaîne hexadécimale
        assert_eq!(rdb_file.length(), 88);
    }

    #[test]
    fn test_to_redis_bytes() {
        let rdb_file = RDBFile::empty_file();
        let bytes = rdb_file.to_redis_bytes();
        // Vérifie que les premiers éléments correspondent au format attendu avec le préfixe "$"
        assert_eq!(bytes[0], b'$');
        // Vérifie que la conversion hexadécimale est correcte pour les premiers octets après le préfixe et la longueur
        // Ici, on teste juste une petite partie pour l'exemple
        assert_eq!(bytes[bytes.len() - 3..], [0xff, 0x5a, 0xa2]);
    }
}
