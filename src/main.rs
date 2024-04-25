// Uncomment this block to pass the first stage
use std::{
    io::{Read, Write},
    net::{TcpListener, TcpStream},
    thread,
};

fn main() {
    // You can use print statements as follows for debugging, they'll be visible when running tests.
    println!("Logs from your program will appear here!");

    // Uncomment this block to pass the first stage
    let listener = TcpListener::bind("127.0.0.1:6379").expect("Can't bind to address");
    let mut threads: Vec<_> = vec![];
    for stream in listener.incoming() {
        match stream {
            Ok(mut stream) => {
                threads.push(thread::spawn(move || handle_client(&mut stream)));
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

fn handle_client(stream: &mut TcpStream) {
    let mut buf = [0; 512];
    while let Ok(n) = stream.read(&mut buf) {
        if n == 0 {
            break;
        }
        println!("Received {n} bytes with data: {:?}", &buf[..n]);
        ping(stream);
    }
}

const PONG: &[u8] = "+PONG\r\n".as_bytes();
fn ping(stream: &mut TcpStream) {
    stream.write_all(PONG).expect("Can't write PONG to stream");
}
