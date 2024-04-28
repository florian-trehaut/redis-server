use std::{
    collections::HashMap,
    io::Error,
    net::TcpListener,
    sync::{Arc, Mutex},
    thread,
};

use crate::{ClientHandler, RedisStore, ServerConfig};

pub mod master;
pub mod slave;

pub trait CreateInstance {
    fn new(config: ServerConfig) -> Self;
}
pub trait RunInstance {
    fn run(&self) -> Result<(), Error>;
}

pub trait ListenInstance {
    fn listen(&self) -> Result<TcpListener, Error>;
}

pub struct RedisInstance {
    config: ServerConfig,
    store: RedisStore,
}

impl RedisInstance {
    fn new(config: ServerConfig) -> Self {
        let store: RedisStore = Arc::new(Mutex::new(HashMap::new()));
        Self { store, config }
    }

    fn _config(&self) -> &ServerConfig {
        &self.config
    }
}

impl ListenInstance for RedisInstance {
    fn listen(&self) -> Result<TcpListener, Error> {
        println!("Listening on port {}", self.config.port());
        let listener = TcpListener::bind(format!("127.0.0.1:{}", self.config.port()))?;
        Ok(listener)
    }
}

impl RunInstance for RedisInstance {
    fn run(&self) -> Result<(), Error> {
        let listener = self.listen()?;
        let mut threads: Vec<_> = vec![];
        for stream in listener.incoming() {
            let mut stream = stream?;
            let store_clone = self.store.clone();
            let config_clone = self.config.clone();
            threads.push(thread::spawn(move || {
                let mut handler = ClientHandler::new(store_clone, config_clone);
                handler.handle(&mut stream);
            }));
        }
        Ok(())
    }
}
