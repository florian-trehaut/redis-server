use crate::{
    redis_commands::RedisCommands,
    redis_info::RedisInfo,
    resp::{Array, RedisResponse, ToRedisBytes},
    server_config::{Offset, ReplicationId},
    ClientHandler, Config, Listen, RedisStore, ReplicaConfig,
};

use super::Run;
use std::{
    collections::HashMap,
    io::{Error, Read, Write},
    net::{TcpListener, TcpStream},
    sync::{Arc, Mutex},
    thread,
};

pub struct ReplicaInstance {
    store: RedisStore,
    config: ReplicaConfig,
    redis_info: Arc<Mutex<RedisInfo>>,
}
impl ClientHandler for ReplicaInstance {}
impl ReplicaInstance {
    #[must_use]
    pub fn new(config: ReplicaConfig) -> Self {
        let store: RedisStore = Arc::new(Mutex::new(HashMap::new()));
        let redis_info = Arc::new(Mutex::new(RedisInfo::new(&Config::Replica(config.clone()))));
        Self {
            store,
            config,
            redis_info,
        }
    }
}

impl Run for ReplicaInstance {
    type Error = Error;
    fn run(&self) -> Result<(), Error> {
        match self.handshake() {
            Ok(()) => println!(
                "Successful handshake with master {}",
                &self.config.replica_of()
            ),
            Err(e) => panic!("{e}"),
        }
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

impl ReplicaInstance {
    fn handshake(&self) -> Result<(), Error> {
        println!(
            "Connecting to master at {}:{}",
            self.config.replica_of().host_address(),
            self.config.replica_of().port()
        );
        let mut stream = TcpStream::connect(format!(
            "{}:{}",
            self.config.replica_of().host_address(),
            self.config.replica_of().port()
        ))?;

        println!("Sending ping...");
        Self::send_ping(&mut stream)?;
        println!("Sending first replconf...");
        self.send_replconf(&mut stream)?;
        Self::send_psync(&mut stream, None, None)?;
        Ok(())
    }

    fn send_ping(stream: &mut TcpStream) -> Result<(), Error> {
        let ping_command = RedisCommands::Ping;
        stream.write_all(&ping_command.to_redis_bytes())?;
        let buf_answer = &mut [0; 1024];
        let n = stream.read(buf_answer)?;

        match RedisResponse::from_bytes(&buf_answer[..n]) {
            RedisResponse::Pong => {
                println!("Master responded with {}", RedisResponse::Ok);
            }
            invalid_answer => {
                eprintln!("Invalid response from master: {invalid_answer}",);
            }
        }
        Ok(())
    }
    fn send_replconf(&self, stream: &mut TcpStream) -> Result<(), Error> {
        let repl_conf_command =
            Array::from_string(&format!("REPLCONF listening-port {}", &self.config.port()));
        println!("Sending to master : '{repl_conf_command}'");
        stream.write_all(&repl_conf_command.to_redis_bytes())?;
        let buf = &mut [0; 1024];
        let n = stream.read(buf)?;
        match RedisResponse::from_bytes(&buf[..n]) {
            RedisResponse::Ok => {
                println!("Master responded REPLCONF with {}", RedisResponse::Ok);
            }
            invalid_answer => {
                eprintln!("Master didn't answer first replconf as expected: {invalid_answer}");
                panic!()
            }
        }
        let repl_conf_command = Array::from_string("REPLCONF capa psync2");
        println!("Sending to master : '{repl_conf_command}'");
        stream.write_all(&repl_conf_command.to_redis_bytes())?;
        let buf = &mut [0; 1024];
        let n = stream.read(buf)?;
        match RedisResponse::from_bytes(&buf[..n]) {
            RedisResponse::Ok => println!(
                "Master responded second replconf with {}",
                RedisResponse::Ok
            ),
            invalid_answer => {
                eprintln!("Master didn't answer second replconf with OK: {invalid_answer}");
                panic!()
            }
        }
        Ok(())
    }
    fn send_psync(
        stream: &mut TcpStream,
        replication_id: Option<ReplicationId>,
        offset: Option<Offset>,
    ) -> Result<(), Error> {
        let replication_id = replication_id.map_or_else(|| ReplicationId::parse(None), |id| id);
        let offset = offset.map_or_else(|| Offset::parse(None), |off| off);
        let command = RedisCommands::Psync(replication_id, offset).to_redis_bytes();
        println!(
            "replica sending PSync command: '{}'",
            String::from_utf8_lossy(&command)
        );
        stream.write_all(&command)?;
        Ok(())
    }
}

impl Listen for ReplicaInstance {
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
