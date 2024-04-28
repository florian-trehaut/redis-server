use redis_starter_rust::{
    CreateInstance, RedisMasterInstance, RedisSlaveInstance, RunInstance, ServerConfig,
};

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let server_config = ServerConfig::from_args(&args);
    match server_config.replica_of() {
        Some(_) => {
            let redis_server = RedisSlaveInstance::new(server_config);
            if let Err(e) = redis_server.run() {
                eprintln!("Error running server: {}", e);
            }
        }
        None => {
            let redis_server = RedisMasterInstance::new(server_config);
            if let Err(e) = redis_server.run() {
                eprintln!("Error running server: {}", e);
            }
        }
    }
}
