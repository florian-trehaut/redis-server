// Uncomment this block to pass the first stage
use std::{
    collections::HashMap,
    net::TcpListener,
    sync::{Arc, Mutex},
    thread,
};

use redis_starter_rust::{ClientHandler, RedisStore};

fn main() {
    // You can use print statements as follows for debugging, they'll be visible when running tests.
    println!("Logs from your program will appear here!");

    let store: RedisStore = Arc::new(Mutex::new(HashMap::new()));
    // Uncomment this block to pass the first stage
    let listener = TcpListener::bind("127.0.0.1:6379").expect("Can't bind to address");
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
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }
    for thread in threads {
        thread.join().expect("Can't join threads");
    }
}
