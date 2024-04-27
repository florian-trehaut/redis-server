/// This is a simple Redis server implementation in Rust.
/// It listens for connections on port 6379 and handles the Redis protocol.
/// It supports the following commands:
/// - SET key value
/// - GET key
/// - PING
/// - ECHO message
/// - SET key value PX milliseconds
use std::{
    collections::HashMap,
    net::TcpListener,
    sync::{Arc, Mutex},
    thread,
};

use redis_starter_rust::{ClientHandler, Host, Port, RedisStore, ReplicaOf, ServerConfig};

fn main() {
    println!("Logs from your program will appear here!");

    let args: Vec<String> = std::env::args().collect();
    let mut port = "6379".to_string(); // default port

    if let Some(port_arg_position) = args.iter().position(|arg| arg == "--port") {
        if args.len() > port_arg_position + 1 {
            port = args[port_arg_position + 1].clone();
        }
    }

    let mut host_of_replica = None;
    let mut port_of_host = None;
    if let Some(replica_arg_position) = args.iter().position(|arg| arg == "--replicaof") {
        match (
            args.get(replica_arg_position + 1),
            args.get(replica_arg_position + 2),
        ) {
            (Some(host), Some(port)) => {
                host_of_replica = Some(host.clone().parse::<Host>().unwrap());
                port_of_host = Some(port.clone().parse::<Port>().unwrap());
            }
            _ => {
                eprintln!("Invalid arguments for --replicaof");
                std::process::exit(1);
            }
        }
    }
    let port: Port = port.parse().expect("Invalid port number");
    let store: RedisStore = Arc::new(Mutex::new(HashMap::new()));

    let replica_of = match (host_of_replica, port_of_host) {
        (Some(host), Some(port)) => Some(ReplicaOf::new(host, port)),
        _ => None,
    };
    let server_config = ServerConfig::new(replica_of);

    let listener = TcpListener::bind(format!("127.0.0.1:{port}")).expect("Can't bind to address");
    let mut threads: Vec<_> = vec![];
    for stream in listener.incoming() {
        match stream {
            Ok(mut stream) => {
                let store_clone = store.clone();
                let server_config = server_config.clone();
                threads.push(thread::spawn(move || {
                    let mut handler = ClientHandler::new(store_clone, server_config);
                    handler.handle(&mut stream);
                }));
            }
            Err(e) => panic!("{e}"),
        }
    }
    for thread in threads {
        thread.join().expect("Can't join threads");
    }
}
