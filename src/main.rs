use clap::Parser;
use tokio::net::{TcpListener, TcpStream};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use std::str;
use reqwest;
use std::env;

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

async fn handle_client(mut stream: TcpStream) {
    println!("New client connected: {}", stream.peer_addr().unwrap());

    // Send a welcome message to the client
    let welcome_message = "Welcome to the elnet server!\nYou are now chatting with an LLM.\nType /help to view available commands.\n> ";
    if let Err(e) = stream.write_all(welcome_message.as_bytes()).await {
        eprintln!("Failed to send welcome message: {}", e);
        return;
    }

    let mut buffer = [0u8; 512];
    loop {
        match stream.read(&mut buffer).await {
            Ok(0) => break, // Connection closed
            Ok(n) => {
                let message = match str::from_utf8(&buffer[..n]) {
                    Ok(v) => v,
                    Err(e) => {
                        eprintln!("Invalid UTF-8 sequence: {}", e);
                        continue;
                    }
                };

                if message.starts_with("/help") {
                    let help_message = "Available commands:\n/help - View available commands\n/quit - Disconnect from the server\n"; // Add more commands later
                    if let Err(e) = stream.write_all(help_message.as_bytes()).await {
                        eprintln!("Failed to send help message: {}", e);
                    }
                } else if message.starts_with("/quit") {
                    println!("Client disconnected: {}", stream.peer_addr().unwrap());
                    break;
                } else {
                    // Call the LLM API with the message
                    let llm_response = call_llm_api(message).await;
                    match llm_response {
                        Ok(response) => {
                            if let Err(e) = stream.write_all(response.as_bytes()).await {
                                eprintln!("Failed to send LLM response: {}", e);
                                break;
                            }
                        }
                        Err(e) => {
                            eprintln!("LLM API call failed: {}", e);
                            break;
                        }
                    }
                    if let Err(e) = stream.write_all("> ".as_bytes()).await {
                        eprintln!("Failed to send prompt: {}", e);
                        break;
                    }
                }
            }
            Err(_) => break,
        }
    }
}

async fn call_llm_api(prompt: &str) -> Result<String, reqwest::Error> {
    let api_key = env::var("GEMINI_API_KEY").expect("GEMINI_API_KEY must be set");
    let url = format!("https://generativelanguage.googleapis.com/v1beta/models/gemini-2.0-flash:generateContent?key={}", api_key);

    let client = reqwest::Client::new();
    let json_body = format!(r#"{{"contents": [{{"parts":[{{"text": "{}"}}]}}]}}"#, prompt);

    let response = client.post(url)
        .header("Content-Type", "application/json")
        .body(json_body)
        .send()
        .await?;

    let response_body = response.text().await?;
    Ok(response_body)
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let args = Args::parse();

    let address = format!("{}:{}", args.hostname, args.port);
    let listener = TcpListener::bind(&address).await?;

    println!("Server listening on {}", address);

    loop {
        let (socket, addr) = listener.accept().await?;
        println!("Accepted connection from {}", addr);
        tokio::spawn(async move {
            handle_client(socket).await;
        });
    }
}