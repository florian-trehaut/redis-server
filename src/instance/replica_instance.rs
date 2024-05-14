use crate::{
    redis_commands::RedisCommands,
    redis_info::RedisInfo,
    resp::{redis_response::RedisResponse, Array, ToRedisBytes},
    server_config::{Offset, ReplicationId},
    ClientHandler, Config, Listen, RedisStore, ReplicaConfig,
};

use super::{client_handler::CommonCommands, Run};
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
impl CommonCommands for ReplicaInstance {
    fn match_redis_command(
        redis_command: RedisCommands,
        stream: &mut TcpStream,
        store: &Arc<Mutex<std::collections::HashMap<String, crate::RedisValue>>>,
        redis_info: &Arc<Mutex<RedisInfo>>,
    ) {
        match redis_command {
            RedisCommands::Ping => Self::ping(stream),
            RedisCommands::Echo(message) => Self::echo(&message, stream),
            RedisCommands::Get(key) => Self::get(store, &key, stream),
            RedisCommands::Set((key, value, expiration)) => {
                Self::set(store, &key, value, expiration, stream);
            }
            RedisCommands::Info(section) => Self::info(redis_info, &section, stream),
            RedisCommands::Replconf(_, _) => {
                Self::respond(&RedisResponse::Ok, stream);
            }
            // RedisCommands::Psync(_, _) => Self::psync(redis_info, stream),
            RedisCommands::FullResync(a, b) => {
                println!(
                    "Received {}",
                    String::from_utf8_lossy(&RedisCommands::FullResync(a, b).to_redis_bytes())
                );
            }
            RedisCommands::Psync(_, _) => Self::respond(&RedisResponse::Null, stream),
        }
    }
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
    fn run(&self) {
        self.handshake();
        let listener = self.listen();
        let mut threads: Vec<_> = vec![];
        for stream in listener.incoming() {
            let mut stream = stream.expect("Replica cannot read stream");
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

impl ReplicaInstance {
    fn handshake(&self) {
        println!(
            "Connecting to master at {}:{}",
            self.config.replica_of().host_address(),
            self.config.replica_of().port()
        );
        let mut stream = TcpStream::connect(format!(
            "{}:{}",
            self.config.replica_of().host_address(),
            self.config.replica_of().port()
        ))
        .expect("Replica cannot connect to master to handshake");

        Self::send_ping(&mut stream);
        self.send_replconf(&mut stream);
        Self::send_psync(&mut stream, None, None);
    }

    fn send_ping(stream: &mut TcpStream) {
        let ping_command = RedisCommands::Ping;
        stream
            .write_all(&ping_command.to_redis_bytes())
            .expect("Replica cannot write ping to master stream");
        let buf_answer = &mut [0; 1024];
        let n = stream
            .read(buf_answer)
            .expect("Replica cannot read ping response from master");

        match RedisResponse::from_bytes(&buf_answer[..n]) {
            RedisResponse::Pong => {
                println!("Master responded with {}", RedisResponse::Ok);
            }
            invalid_answer => {
                panic!("Invalid response from master: {invalid_answer}");
            }
        }
    }
    fn send_replconf(&self, stream: &mut TcpStream) {
        let repl_conf_command =
            Array::from_string(&format!("REPLCONF listening-port {}", &self.config.port()));
        println!("Sending to master : '{repl_conf_command}'");
        stream
            .write_all(&repl_conf_command.to_redis_bytes())
            .expect("Replica cannot write replconf to master stream");
        let buf = &mut [0; 1024];
        let n = stream.read(buf).expect("Replica ");
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
        stream
            .write_all(&repl_conf_command.to_redis_bytes())
            .expect("Replica cannot write second replconf to master stream");
        let buf = &mut [0; 1024];
        let n = stream
            .read(buf)
            .expect("Replica cannot read second replconf response from master");
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
    }
    fn send_psync(
        stream: &mut TcpStream,
        replication_id: Option<ReplicationId>,
        offset: Option<Offset>,
    ) {
        let replication_id = replication_id.map_or_else(|| ReplicationId::parse(None), |id| id);
        let offset = offset.map_or_else(|| Offset::parse(None), |off| off);
        let command = RedisCommands::Psync(replication_id, offset).to_redis_bytes();
        println!(
            "replica sending PSync command: '{}'",
            String::from_utf8_lossy(&command)
        );
        stream
            .write_all(&command)
            .expect("Replica cannot write psync to master stream");
        println!("Psync command sent");
        let buf = &mut [0; 1024];
        if let Ok(n) = stream.read(buf) {
            println!(
                "Master responded with {}",
                String::from_utf8_lossy(&buf[..n])
            );
        } else {
            eprintln!("Master didn't responded to psync bro");
            panic!();
        }
    }
}

impl Listen for ReplicaInstance {
    /// Listens to incoming connections and returns a `TcpListener`.
    ///
    /// # Returns
    ///
    /// Returns a `TcpListener` if the listening is successful, otherwise returns an `Error`.
    type Error = Error;
    fn listen(&self) -> TcpListener {
        println!("Listening on port {}", self.config.port());

        TcpListener::bind(format!("127.0.0.1:{}", self.config.port()))
            .expect("Replica cannot listen")
    }
}
