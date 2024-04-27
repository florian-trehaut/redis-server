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

use redis_starter_rust::{ClientHandler, RedisStore};

fn main() {
    println!("Logs from your program will appear here!");

    let args: Vec<String> = std::env::args().collect();
    let mut port = "6379".to_string(); // default port

    if let Some(port_arg_position) = args.iter().position(|arg| arg == "--port") {
        if args.len() > port_arg_position + 1 {
            port = args[port_arg_position + 1].clone();
        }
    }

    let port: u16 = port.parse().expect("Invalid port number");
    let store: RedisStore = Arc::new(Mutex::new(HashMap::new()));
    let listener = TcpListener::bind(format!("127.0.0.1:{port}")).expect("Can't bind to address");
    let mut threads: Vec<_> = vec![];
    for stream in listener.incoming() {
        match stream {
            Ok(mut stream) => {
                let store_clone = store.clone();
                threads.push(thread::spawn(move || {
                    let mut handler = ClientHandler::new(store_clone);
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
