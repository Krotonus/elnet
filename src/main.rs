use clap::Parser;
use tokio::net::{TcpListener, TcpStream};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use std::str;
use reqwest;
use std::env;
use serde::{Deserialize, Serialize};
use serde_json;

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

#[derive(Serialize, Deserialize, Debug)]
struct LlmResponse {
    candidates: Option<Vec<Candidate>>,
    usage_metadata: Option<UsageMetadata>,
    model_version: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
struct Candidate {
    content: Option<Content>,
    finish_reason: Option<String>,
    avg_logprobs: Option<f64>,
}

#[derive(Serialize, Deserialize, Debug)]
struct Content {
    parts: Option<Vec<Part>>,
    role: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
struct Part {
    text: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
struct UsageMetadata {
    prompt_token_count: Option<i32>,
    candidates_token_count: Option<i32>,
    total_token_count: Option<i32>,
    prompt_tokens_details: Option<Vec<TokenDetails>>,
    candidates_tokens_details: Option<Vec<TokenDetails>>,
}

#[derive(Serialize, Deserialize, Debug)]
struct TokenDetails {
    modality: Option<String>,
    token_count: Option<i32>,
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
    let llm_response: Result<LlmResponse, serde_json::Error> = serde_json::from_str(&response_body);

    match llm_response {
        Ok(llm_response) => {
            let response_text = llm_response.candidates
                .and_then(|candidates| candidates.into_iter().next())
                .and_then(|candidate| candidate.content)
                .and_then(|content| content.parts)
                .and_then(|parts| parts.into_iter().next())
                .and_then(|part| part.text)
                .unwrap_or_else(|| "No response from LLM".to_string());
            Ok(response_text)
        }
        Err(e) => {
            eprintln!("Failed to parse LLM response: {}", e);
            Ok("Failed to parse LLM response".to_string())
        }
    }
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