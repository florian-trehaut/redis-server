use redis_starter_rust::{Create, RedisMasterInstance, RedisSlaveInstance, Run, ServerConfig};

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let server_config =
        match ServerConfig::from_args(&args.iter().map(String::as_str).collect::<Vec<&str>>()) {
            Ok(config) => config,
            Err(e) => {
                eprintln!("Error parsing server configuration: {e}");
                return;
            }
        };
    match server_config {
        ServerConfig::Slave(config) => {
            let config = ServerConfig::Slave(config);
            let redis_server =
                RedisSlaveInstance::new(config.clone()).expect("Can't build slave instance");
            if let Err(e) = redis_server.run(config) {
                eprintln!("Error running server: {e}");
            }
        }
        ServerConfig::Master(config) => {
            let config = ServerConfig::Master(config);
            let redis_server =
                RedisMasterInstance::new(config.clone()).expect("Can't build master instance");
            if let Err(e) = redis_server.run(config) {
                eprintln!("Error running server: {e}");
            }
        }
    }
}
