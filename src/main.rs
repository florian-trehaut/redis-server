use redis_starter_rust::{Config, MasterInstance, ReplicaInstance, Run};

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let server_config =
        match Config::from_args(&args.iter().map(String::as_str).collect::<Vec<&str>>()) {
            Ok(config) => config,
            Err(e) => {
                eprintln!("Error parsing server configuration: {e}");
                return;
            }
        };
    match server_config {
        Config::Replica(config) => {
            let redis_server = ReplicaInstance::new(config);
            if let Err(e) = redis_server.run() {
                eprintln!("Error running server: {e}");
            }
        }
        Config::Master(config) => {
            let redis_server = MasterInstance::new(config);
            if let Err(e) = redis_server.run() {
                eprintln!("Error running server: {e}");
            }
        }
    }
}
