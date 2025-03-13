use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::thread;

fn handle_client(mut stream: TcpStream) {
    let mut buffer = [0; 512];
    loop {
        match stream.read(&mut buffer) {
            Ok(0) => break, // Connection closed
            Ok(n) => {
                if stream.write_all(&buffer[0..n]).is_err() {
                    break;
                }
            }
            Err(_) => break,
        }
    }
}

fn main() -> std::io::Result<()> {
    let listener = TcpListener::bind("0.0.0.0:8080")?;
    println!("Server listening on port 8080");

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                thread::spawn(|| handle_client(stream));
            }
            Err(e) => {
                eprintln!("Connection failed: {}", e);
            }
        }
    }

    Ok(())
}