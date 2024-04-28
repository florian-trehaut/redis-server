use crate::{resp::BulkString, ServerConfig};

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
        self.handshake()
            .expect("Error in handshake with master instance. Exiting...");
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
        self.send_ping()?;
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

        let ping_command = "*1\r\n$4\r\nPING\r\n";
        stream.write_all(ping_command.as_bytes())?;
        let buf_answer = &mut [0; 1024];
        let n = stream.read(buf_answer)?;

        match BulkString::from_bytes(&buf_answer[..n]) {
            Ok(_) => Ok(()),
            Err(e) => {
                eprintln!(
                    "Error parsing buf: {} \n error: {:?}",
                    std::str::from_utf8(&buf_answer[..n]).unwrap(),
                    e
                );
                panic!()
            }
        }
    }
}
