use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::thread;
use clap::Parser;

#[derive(Parser, Debug)]
#[clap(author = "Krotonus", version = "0.0.1", about = "A simple TCP server that connects to an LLM.", long_about = None)]
struct Args {
    /// Sets the hostname to listen on
    #[clap(long, default_value = "0.0.0.0")]
    hostname: String,

    /// Sets the port to listen on
    #[clap(long, default_value = "8080")]
    port: String,
}

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
    let args = Args::parse();

    let address = format!("{}:{}", args.hostname, args.port);
    let listener = TcpListener::bind(&address)?;

    println!("Server listening on {}", address);

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