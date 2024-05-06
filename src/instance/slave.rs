use crate::{
    resp::{Array, RedisResponse, ToRedisBytes},
    server_config::server_config::SlaveConfigError,
    ServerConfig, SlaveConfig,
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
    fn new(config: ServerConfig) -> Result<Self, Self::ConfigError> {
        let instance = Redis::new();
        let config = SlaveConfig::from_server_config(config)?;
        Ok(Self { instance, config })
    }
}

impl Run for RedisSlaveInstance {
    type Error = Error;
    fn run(&self, config: ServerConfig) -> Result<(), Error> {
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
        println!("Sending ping...");
        self.send_ping()?;
        println!("Sending first replconf...");
        self.send_replconf()?;
        Ok(())
    }

    fn send_ping(&self) -> Result<(), Error> {
        let mut stream = TcpStream::connect(format!(
            "{}:{}",
            self.config.replica_of().host_address().clone(),
            self.config.replica_of().port().clone()
        ))?;

        let ping_command = "*1\r\n$4\r\nPING\r\n"; // Todo: To struct
        stream.write_all(ping_command.as_bytes())?;
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
    fn send_replconf(&self) -> Result<(), Error> {
        let repl_conf_command = Array::from_string(&format!(
            "REPLCONF listening-port {}",
            &self.config.replica_of().port()
        ));
        let replica_config = self.config.replica_of();
        let mut stream = TcpStream::connect(format!(
            "{}:{}",
            replica_config.host_address(),
            replica_config.port()
        ))?;
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
}
