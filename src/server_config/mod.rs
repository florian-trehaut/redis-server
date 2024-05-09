use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use crate::RedisValue;

pub mod host;
pub mod port;
pub mod replica;
pub mod server;
pub type RedisStore = Arc<Mutex<HashMap<String, RedisValue>>>;
