use std::fmt::Display;

#[derive(Clone, Debug)]
pub struct SimpleString {
    data: String,
}
impl SimpleString {
    pub fn data(&self) -> &str {
        &self.data
    }
    pub fn _from_string(s: &str) -> Self {
        Self {
            data: s.to_string(),
        }
    }
    pub fn from_bytes(buf: &[u8]) -> Result<Self, SimpleStringError> {
        let string = std::str::from_utf8(buf)?;
        if string.starts_with('+') {
            Ok(Self {
                data: string.replace('+', ""),
            })
        } else {
            Err(SimpleStringError::InvalidSimpleString)
        }
    }
}

pub enum SimpleStringError {
    Utf8Error(std::str::Utf8Error),
    InvalidSimpleString,
}
impl From<std::str::Utf8Error> for SimpleStringError {
    fn from(err: std::str::Utf8Error) -> Self {
        Self::Utf8Error(err)
    }
}
impl Display for SimpleStringError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Utf8Error(err) => write!(f, "{err}"),
            Self::InvalidSimpleString => write!(f, "Invalid simple string doesn't start with '+'"),
        }
    }
}
