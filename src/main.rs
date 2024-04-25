// Uncomment this block to pass the first stage
use std::{
    io::{Read, Write},
    net::{TcpListener, TcpStream},
};

fn main() {
    // You can use print statements as follows for debugging, they'll be visible when running tests.
    println!("Logs from your program will appear here!");

    // Uncomment this block to pass the first stage

    let listener = TcpListener::bind("127.0.0.1:6379").unwrap();

    for stream in listener.incoming() {
        match stream {
            Ok(mut stream) => handle_client(&mut stream),
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }
}

fn handle_client(stream: &mut TcpStream) {
    let mut buf = [0; 512];
    let n = stream.read(&mut buf).unwrap();
    println!("Received {n} bytes with data: {:?}", &buf[..n]);
    ping(stream);
}

const PONG: &str = "+PONG\r\n";
fn ping(stream: &mut TcpStream) {
    let mut buf = PONG.as_bytes();
    stream.write_all(&mut buf).unwrap();
}
