use clap::Parser;
use tokio::net::TcpListener;
use tokio::spawn;
use std::net::TcpStream as StdTcpStream;
use tokio::net::TcpStream;
mod client_handler;
mod llm;
mod tui;
mod ws_handler;

#[derive(Parser, Debug)]
#[clap(author = "Krotonus", version = "0.0.1", about = "A simple WebSocket server that connects to an LLM.", long_about = None)]
struct Args {
    /// Sets the hostname to listen on
    #[clap(long, default_value = "0.0.0.0")]
    hostname: String,

    /// Sets the port to listen on
    #[clap(long, default_value = "8080")]
    port: String,
}

const INDEX_HTML: &str = include_str!("../static/index.html");

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let args = Args::parse();
    let address = format!("{}:{}", args.hostname, args.port);
    let listener = TcpListener::bind(&address).await?;

    println!("Server listening on {}", address);

    loop {
        let (socket, addr) = listener.accept().await?;
        println!("Accepted connection from {}", addr);
        spawn(async move {
            let std_socket = match socket.into_std() {
                Ok(s) => s,
                Err(e) => {
                    eprintln!("Failed to convert socket: {}", e);
                    return;
                }
            };
            let socket_clone = match StdTcpStream::try_clone(&std_socket) {
                Ok(s) => s,
                Err(e) => {
                    eprintln!("Failed to clone socket: {}", e);
                    return;
                }
            };
            let mut stream = match TcpStream::from_std(socket_clone) {
                Ok(s) => s,
                Err(e) => {
                    eprintln!("Failed to create stream: {}", e);
                    return;
                }
            };
            let socket = match TcpStream::from_std(std_socket) {
                Ok(s) => s,
                Err(e) => {
                    eprintln!("Failed to create socket: {}", e);
                    return;
                }
            };

            let mut buffer = [0; 1024];
            if let Ok(n) = stream.peek(&mut buffer).await {
                let data = String::from_utf8_lossy(&buffer[..n]);
                if data.contains("GET /ws") {
                    ws_handler::handle_websocket(socket).await;
                } else if data.contains("GET / ") {
                    let response = format!(
                        "HTTP/1.1 200 OK\r\nContent-Type: text/html\r\nContent-Length: {}\r\n\r\n{}",
                        INDEX_HTML.len(),
                        INDEX_HTML
                    );
                    let _ = stream.try_write(response.as_bytes());
                }
            }
        });
    }
}