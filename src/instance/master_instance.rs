use crate::{redis_info::RedisInfo, ClientHandler, Config, Listen, MasterConfig, RedisStore};

use super::Run;
use std::{
    collections::HashMap,
    io::Error,
    net::TcpListener,
    sync::{Arc, Mutex},
    thread,
};

pub struct MasterInstance {
    store: RedisStore,
    config: MasterConfig,
    redis_info: Arc<Mutex<RedisInfo>>,
}
impl ClientHandler for MasterInstance {}
impl MasterInstance {
    #[must_use]
    pub fn new(config: MasterConfig) -> Self {
        let store: RedisStore = Arc::new(Mutex::new(HashMap::new()));
        let redis_info = Arc::new(Mutex::new(RedisInfo::new(&Config::Master(config.clone()))));
        Self {
            store,
            config,
            redis_info,
        }
    }
}

impl Run for MasterInstance {
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

impl Listen for MasterInstance {
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
