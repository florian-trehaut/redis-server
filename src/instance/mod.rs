use std::{
    collections::HashMap,
    io::Error,
    net::TcpListener,
    sync::{Arc, Mutex},
    thread,
};

use crate::{ClientHandler, Config, RedisStore};

pub mod master;
pub mod slave;

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
    fn run(&self, config: Config) -> Result<(), Self::Error>;
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
    fn listen(&self, config: Config) -> Result<TcpListener, Self::Error>;
}

/// Represents a Redis instance.
pub struct Redis {
    store: RedisStore,
}

impl Redis {
    /// Creates a new Redis instance with the given configuration.
    fn new() -> Self {
        let store: RedisStore = Arc::new(Mutex::new(HashMap::new()));
        Self { store }
    }
}

impl Listen for Redis {
    /// Listens to incoming connections and returns a `TcpListener`.
    ///
    /// # Returns
    ///
    /// Returns a `TcpListener` if the listening is successful, otherwise returns an `Error`.
    type Error = Error;
    fn listen(&self, config: Config) -> Result<TcpListener, Error> {
        println!("Listening on port {}", config.port());
        let listener = TcpListener::bind(format!("127.0.0.1:{}", config.port()))?;
        Ok(listener)
    }
}

impl Run for Redis {
    /// Runs the Redis instance.
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` if the instance runs successfully, otherwise returns an `Error`.
    type Error = Error;
    fn run(&self, config: Config) -> Result<(), Error> {
        let listener = self.listen(config.clone())?;
        let mut threads: Vec<_> = vec![];
        for stream in listener.incoming() {
            let mut stream = stream?;
            let store_clone = self.store.clone();
            let config_clone = config.clone();
            threads.push(thread::spawn(move || {
                let mut handler = ClientHandler::new(store_clone, &config_clone);
                handler.handle(&mut stream);
            }));
        }
        for handle in threads {
            handle.join().expect("Panic occurred in thread");
        }
        Ok(())
    }
}
