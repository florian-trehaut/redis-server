use crate::{server_config::server_config::MasterConfigError, MasterConfig, ServerConfig};

use super::{Create, Redis, Run};
use std::io::Error;

pub struct RedisMasterInstance {
    instance: Redis,
    config: MasterConfig,
}

impl Create for RedisMasterInstance {
    type Instance = Self;
    type ConfigError = MasterConfigError;
    fn new(args: ServerConfig) -> Result<Self, MasterConfigError> {
        let instance = Redis::new();
        let config = MasterConfig::from_server_config(args)?;
        Ok(Self { instance, config })
    }
}

impl Run for RedisMasterInstance {
    type Error = Error;
    fn run(&self, config: ServerConfig) -> Result<(), Error> {
        self.instance.run(config)
    }
}
