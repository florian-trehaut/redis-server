use std::{
    collections::HashMap,
    fmt::Display,
    sync::{Arc, Mutex},
};

use crate::RedisValue;

pub mod host;
pub mod port;
pub mod replica;
pub mod server;
pub type RedisStore = Arc<Mutex<HashMap<String, RedisValue>>>;

#[derive(Debug, PartialEq, Eq)]
pub struct ReplicationId(String);
impl ReplicationId {
    pub fn len(&self) -> usize {
        self.0.len()
    }
    pub fn parse(id: Option<String>) -> Self {
        match id {
            Some(id) => ReplicationId(id),
            None => ReplicationId("?".to_string()),
        }
    }
}
impl Display for ReplicationId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct Offset(i8);
impl Offset {
    pub fn len(&self) -> usize {
        self.0.to_string().len()
    }
    pub fn parse(offset: Option<i8>) -> Self {
        match offset {
            Some(offset) => Offset(offset),
            None => Offset(-1),
        }
    }
}
impl Display for Offset {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
