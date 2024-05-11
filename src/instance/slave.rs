use crate::{
    redis_commands::RedisCommands,
    resp::{Array, RedisResponse, ToRedisBytes},
    server_config::{server::SlaveConfigError, Offset, ReplicationId},
    Config, SlaveConfig,
};

use super::{Create, Redis, Run};
use std::{
    io::{Error, Read, Write},
    net::TcpStream,
};

pub struct RedisSlaveInstance {
    instance: Redis,
    config: SlaveConfig,
}

impl Create for RedisSlaveInstance {
    type Instance = Self;
    type ConfigError = SlaveConfigError;
    fn new(config: Config) -> Result<Self, Self::ConfigError> {
        let instance = Redis::new();
        let config = SlaveConfig::from_server_config(config)?;
        Ok(Self { instance, config })
    }
}

impl Run for RedisSlaveInstance {
    type Error = Error;
    fn run(&self, config: Config) -> Result<(), Error> {
        match self.handshake() {
            Ok(()) => println!(
                "Successful handshake with master {}",
                &self.config.replica_of()
            ),
            Err(e) => panic!("{e}"),
        }
        self.instance.run(config)?;
        Ok(())
    }
}

impl RedisSlaveInstance {
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
            "Slave sending PSync command: '{}'",
            String::from_utf8_lossy(&command)
        );
        stream.write_all(&command)?;
        Ok(())
    }
}
