use std::{
    io::Error,
    net::TcpListener,
    sync::{Arc, Mutex},
    thread,
};

use crate::{redis_info::RedisInfo, ClientHandler, Config, RedisStore};

#[allow(clippy::module_name_repetitions)]
pub mod master_instance;
#[allow(clippy::module_name_repetitions)]
pub mod replica_instance;

pub mod client_handler;

/// Trait for creating a Redis instance.
pub trait Create {
    type Instance;
    type ConfigError;
    /// Creates a new Redis instance with the given configuration.
    ///
    /// # Returns
    ///
    /// Returns a `Result` containing the instance if the creation is successful, otherwise returns a `ConfigError`.
    ///
    /// # Errors
    ///
    /// If the instance fails to be created, a `ConfigError` is returned.
    fn new(config: Config) -> Result<Self::Instance, Self::ConfigError>;
}

/// Trait for running a Redis instance.
pub trait Run {
    type Error;
    /// Runs the Redis instance.
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` if the instance runs successfully, otherwise returns an `Error`.
    ///
    /// # Errors
    ///
    /// If the instance fails to run, an `Error` is returned.
    fn run(&self) -> Result<(), Self::Error>;
}

/// Trait for listening to incoming connections.
pub trait Listen {
    type Error;
    /// Listens to incoming connections and returns a `TcpListener`.
    ///
    /// # Returns
    ///
    /// Returns a `TcpListener` if the listening is successful, otherwise returns an `Error`.
    ///
    /// # Errors
    ///
    /// If the listener fails to bind to the address, an `Error` is returned.
    fn listen(&self) -> Result<TcpListener, Self::Error>;
}

/// Represents a Redis instance.
pub struct Instance {
    store: RedisStore,
    config: Config,
    redis_info: Arc<Mutex<RedisInfo>>,
}
impl ClientHandler for Instance {}

impl Listen for Instance {
    /// Listens to incoming connections and returns a `TcpListener`.
    ///
    /// # Returns
    ///
    /// Returns a `TcpListener` if the listening is successful, otherwise returns an `Error`.
    type Error = Error;
    fn listen(&self) -> Result<TcpListener, Error> {
        println!("Listening on port {}", self.config.port());
        let listener = TcpListener::bind(format!("127.0.0.1:{}", self.config.port()))?;
        Ok(listener)
    }
}

impl Run for Instance {
    /// Runs the Redis instance.
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` if the instance runs successfully, otherwise returns an `Error`.
    type Error = Error;
    fn run(&self) -> Result<(), Error> {
        let listener = self.listen()?;
        let mut threads: Vec<_> = vec![];
        for stream in listener.incoming() {
            let mut stream = stream?;
            let store = self.store.clone();
            let redis_info = self.redis_info.clone();
            threads.push(thread::spawn(move || {
                Self::handle(redis_info, store, &mut stream);
            }));
        }
        for handle in threads {
            handle.join().expect("Panic occurred in thread");
        }
        Ok(())
    }
}
