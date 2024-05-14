use crate::{
    redis_commands::RedisCommands, redis_info::RedisInfo, resp::redis_response::RedisResponse,
    ClientHandler, Config, Listen, MasterConfig, RedisStore,
};

use super::{client_handler::CommonCommands, Run};
use std::{
    collections::HashMap,
    io::Error,
    net::{TcpListener, TcpStream},
    sync::{Arc, Mutex},
    thread,
};

pub struct MasterInstance {
    store: RedisStore,
    config: MasterConfig,
    redis_info: Arc<Mutex<RedisInfo>>,
}
impl CommonCommands for MasterInstance {
    fn match_redis_command(
        redis_command: crate::redis_commands::RedisCommands,
        stream: &mut std::net::TcpStream,
        store: &Arc<Mutex<std::collections::HashMap<String, crate::RedisValue>>>,
        redis_info: &Arc<Mutex<RedisInfo>>,
    ) {
        match &redis_command {
            RedisCommands::Ping => Self::ping(stream),
            RedisCommands::Echo(message) => Self::echo(message, stream),
            RedisCommands::Get(key) => Self::get(store, key, stream),
            RedisCommands::Set((key, value, expiration)) => {
                Self::set(store, key, value.clone(), expiration.to_owned(), stream);
            }
            RedisCommands::Info(section) => Self::info(redis_info, section, stream),
            RedisCommands::Replconf(_, _) => {
                Self::respond(&RedisResponse::Ok, stream);
            }
            RedisCommands::Psync(_, _) => Self::psync(redis_info, stream),
            command => unimplemented!("{command} is unimplemented for Master"),
        }
        println!("Matched command '{redis_command}'");
    }
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
    fn psync(server_info: &Arc<Mutex<RedisInfo>>, stream: &mut TcpStream) {
        println!("Received PYSNC command");

        let replid;
        let offset;
        {
            let server_info_locked = server_info
                .lock()
                .expect("Poisonned store when opening server info");
            replid = server_info_locked.master_replid().to_owned();
            offset = server_info_locked.master_repl_offset().to_owned();
        }
        let command = RedisCommands::FullResync(replid, offset);
        println!("Unlocked redis_info");
        Self::respond(&command, stream);
    }
}

impl Run for MasterInstance {
    /// Runs the Redis instance.
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` if the instance runs successfully, otherwise returns an `Error`.
    type Error = Error;
    fn run(&self) {
        let listener = self.listen();
        let mut threads: Vec<_> = vec![];
        for stream in listener.incoming() {
            let mut stream = stream.expect("Cannot read stream in master");
            let store = self.store.clone();
            let redis_info = self.redis_info.clone();
            threads.push(thread::spawn(move || {
                Self::handle(redis_info, store, &mut stream);
            }));
        }
        for handle in threads {
            handle.join().expect("Panic occurred in thread");
        }
    }
}

impl Listen for MasterInstance {
    /// Listens to incoming connections and returns a `TcpListener`.
    ///
    /// # Returns
    ///
    /// Returns a `TcpListener` if the listening is successful, otherwise returns an `Error`.
    type Error = Error;
    fn listen(&self) -> TcpListener {
        println!("Listening on port {}", self.config.port());

        TcpListener::bind(format!("127.0.0.1:{}", self.config.port()))
            .expect("Master cannot listen")
    }
}
