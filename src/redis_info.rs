use std::fmt::Display;

use crate::{
    resp::BulkString,
    server_config::{Offset, ReplicationId},
    Config,
};

#[derive(Debug, Clone)]
pub struct RedisInfo {
    role: Role,
    master_replid: ReplicationId,
    master_repl_offset: Offset,
}
impl RedisInfo {
    pub fn new(server_config: &Config) -> Self {
        match server_config {
            Config::Master(_) => Self {
                role: Role::Master,
                master_replid: ReplicationId::parse(Some("Master".to_string())),
                master_repl_offset: Offset::parse(Some(0)),
            },
            Config::Replica(_) => Self {
                role: Role::Replica,
                master_replid: ReplicationId::parse(None),
                master_repl_offset: Offset::parse(None),
            },
        }
    }
    pub fn to_bulk_string(&self) -> BulkString {
        BulkString::from(format!("{self}").as_str())
    }
    pub const fn master_replid(&self) -> &ReplicationId {
        &self.master_replid
    }
    pub const fn master_repl_offset(&self) -> &Offset {
        &self.master_repl_offset
    }
    pub const fn role(&self) -> &Role {
        &self.role
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Role {
    Master,
    Replica,
}
impl Display for Role {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Master => write!(f, "master"),
            Self::Replica => write!(f, "slave"),
        }
    }
}
