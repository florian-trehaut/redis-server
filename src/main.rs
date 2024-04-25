// Uncomment this block to pass the first stage
use std::{
    io::{Read, Write},
    net::{TcpListener, TcpStream},
};

fn main() {
    // You can use print statements as follows for debugging, they'll be visible when running tests.
    println!("Logs from your program will appear here!");

    // Uncomment this block to pass the first stage
    match TcpListener::bind("127.0.0.1:6379") {
        Ok(listener) => {
            for stream in listener.incoming() {
                match stream {
                    Ok(mut stream) => handle_client(&mut stream),
                    Err(e) => {
                        println!("error: {}", e);
                    }
                }
            }
        }
        Err(e) => {
            eprintln!("Can't bind to address");
            panic!("{e}");
        }
    };
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{Read, Write};
    use std::net::TcpStream;
    use std::thread;

    fn test_command(command: &[u8], expected_response: &[u8]) {
        // Start the server in a new thread
        thread::spawn(main);

        // Give the server a little time to start
        thread::sleep(std::time::Duration::from_secs(1));

        // Connect to the server
        let mut stream = TcpStream::connect("127.0.0.1:6379").unwrap();

        // Send a command
        stream.write_all(command).unwrap();

        // Read the response
        let mut buf = [0; 512];
        let n = stream.read(&mut buf).unwrap();

        // Check that the response is correct
        assert_eq!(&buf[..n], expected_response);
    }

    #[test]
    fn test_ping() {
        test_command(b"ping\r\n\n\n", b"+PONG\r\n");
    }
    #[test]
    fn test_ping_ping() {
        test_command(b"ping\r\n\nping\r\n\n", b"+PONG\r\n+PONG\r\n");
    }

    // Add more tests here...
}
