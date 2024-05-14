/// Represents a simple string in RESP protocol
/// A simple string is a string prefixed with '+'
#[derive(Clone, Debug)]
pub struct SimpleString {
    data: String,
}
impl SimpleString {
    pub fn data(&self) -> &str {
        &self.data
    }

    pub fn from_bytes(buf: &[u8]) -> Self {
        let string = String::from_utf8_lossy(buf);
        if string.starts_with('+') {
            Self {
                data: string.replace('+', ""),
            }
        } else {
            panic!("Invalid simplestring: '{string}'")
        }
    }
}
