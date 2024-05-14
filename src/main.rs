use redis_starter_rust::{Config, MasterInstance, ReplicaInstance, Run};

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let server_config = Config::from_args(&args.iter().map(String::as_str).collect::<Vec<&str>>());
    match server_config {
        Config::Replica(config) => {
            let redis_server = ReplicaInstance::new(config);
            redis_server.run();
        }
        Config::Master(config) => {
            let redis_server = MasterInstance::new(config);
            redis_server.run();
        }
    }
}
