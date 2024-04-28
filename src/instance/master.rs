use crate::ServerConfig;

use super::{CreateInstance, RedisInstance, RunInstance};
use std::io::Error;

pub struct RedisMasterInstance {
    instance: RedisInstance,
}

impl CreateInstance for RedisMasterInstance {
    fn new(config: ServerConfig) -> Self {
        Self {
            instance: RedisInstance::new(config),
        }
    }
}

impl RunInstance for RedisMasterInstance {
    fn run(&self) -> Result<(), Error> {
        self.instance.run()
    }
}
