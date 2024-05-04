use crate::{
    resp::{RedisResponse, RespArray, ToRedisBytes},
    ServerConfig,
};

use super::{CreateInstance, RedisInstance, RunInstance};
use std::{
    io::{Error, Read, Write},
    net::TcpStream,
};

pub struct RedisSlaveInstance {
    instance: RedisInstance,
}

impl CreateInstance for RedisSlaveInstance {
    fn new(config: ServerConfig) -> Self {
        Self {
            instance: RedisInstance::new(config),
        }
    }
}

impl RunInstance for RedisSlaveInstance {
    fn run(&self) -> Result<(), Error> {
        match self.handshake() {
            Ok(_) => println!(
                "Successful handshake with master {}",
                &self.instance.config.replica_of().unwrap()
            ),
            Err(e) => panic!("{e}"),
        }
        self.instance.run()?;
        Ok(())
    }
}

impl RedisSlaveInstance {
    fn handshake(&self) -> Result<(), Error> {
        println!(
            "Connecting to master at {}:{}",
            self.instance.config.replica_of().unwrap().host_address(),
            self.instance.config.replica_of().unwrap().port()
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
            self.instance
                .config
                .replica_of()
                .unwrap()
                .host_address()
                .clone(),
            self.instance.config.replica_of().unwrap().port().clone()
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
        let repl_conf_command = RespArray::from_string(&format!(
            "REPLCONF listening-port {}",
            &self.instance.config.replica_of().unwrap().port()
        ));
        let replica_config = self.instance._config().replica_of().unwrap();
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
        let replconf_command = RespArray::from_string("REPLCONF capa psync2");
        stream.write_all(&replconf_command.to_redis_bytes())?;
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
