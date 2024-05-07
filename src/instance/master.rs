use crate::{server_config::server::MasterConfigError, Config, MasterConfig};

use super::{Create, Redis, Run};
use std::io::Error;

pub struct RedisMasterInstance {
    instance: Redis,
    config: MasterConfig,
}

impl Create for RedisMasterInstance {
    type Instance = Self;
    type ConfigError = MasterConfigError;
    fn new(args: Config) -> Result<Self, MasterConfigError> {
        let instance = Redis::new();
        let config = MasterConfig::from_server_config(args)?;
        Ok(Self { instance, config })
    }
}

impl Run for RedisMasterInstance {
    type Error = Error;
    fn run(&self, _config: Config) -> Result<(), Error> {
        self.instance.run(Config::Master(self.config.clone()))
    }
}
