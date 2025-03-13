use clap::Parser;
use tokio::net::TcpListener;
use tokio::spawn;
mod client_handler;
mod llm;

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
            client_handler::handle_client(socket).await;
        });
    }
}