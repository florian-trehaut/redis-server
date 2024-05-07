use std::fmt::Display;

use crate::{resp::BulkString, Config};

pub struct RedisInfo {
    role: Role,
    master_replid: Id,
    master_repl_offset: Offset,
}
impl RedisInfo {
    pub fn new(server_config: &Config) -> Self {
        let role = match server_config {
            Config::Master(_) => Role::Master,
            Config::Slave(_) => Role::Slave,
        };
        Self {
            role,
            master_replid: Id::new(),
            master_repl_offset: Offset::new(),
        }
    }
    pub fn to_bulk_string(&self) -> BulkString {
        BulkString::from_string(format!("{self}").as_str())
    }
}
impl Display for RedisInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "role:{}\r\n", self.role)?;
        write!(f, "master_replid:{}\r\n", self.master_replid)?;
        write!(f, "master_repl_offset:{}\r\n", self.master_repl_offset)?;
        Ok(())
    }
}

enum Role {
    Master,
    Slave,
}
impl Display for Role {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Master => write!(f, "master"),
            Self::Slave => write!(f, "slave"),
        }
    }
}
#[derive(Debug, Clone)]
struct Id(String);
impl Id {
    fn new() -> Self {
        Self("id".to_string())
    }
    fn get(&self) -> &str {
        &self.0
    }
}
impl Display for Id {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.get())
    }
}
#[derive(Debug, Clone)]
struct Offset(u64);
impl Offset {
    const fn new() -> Self {
        Self(0)
    }
    const fn get(&self) -> u64 {
        self.0
    }
}
impl Display for Offset {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.get())
    }
}
