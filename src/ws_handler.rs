use futures_util::{SinkExt, StreamExt};
use tokio::net::TcpStream;
use tokio_tungstenite::{
    accept_async,
    tungstenite::Message,
    WebSocketStream,
};
use crate::llm::call_llm_api;

pub async fn handle_websocket(stream: TcpStream) {
    let ws_stream = match accept_async(stream).await {
        Ok(ws) => ws,
        Err(e) => {
            eprintln!("WebSocket handshake failed: {}", e);
            return;
        }
    };

    handle_websocket_connection(ws_stream).await;
}

async fn handle_websocket_connection(mut ws_stream: WebSocketStream<TcpStream>) {
    while let Some(msg) = ws_stream.next().await {
        match msg {
            Ok(Message::Text(text)) => {
                // Handle incoming message
                match call_llm_api(&text).await {
                    Ok(response) => {
                        if let Err(e) = ws_stream.send(Message::Text(response)).await {
                            eprintln!("Error sending message: {}", e);
                            break;
                        }
                    }
                    Err(e) => {
                        eprintln!("Error calling LLM API: {}", e);
                        if let Err(e) = ws_stream
                            .send(Message::Text("Error processing your request".to_string()))
                            .await
                        {
                            eprintln!("Error sending error message: {}", e);
                            break;
                        }
                    }
                }
            }
            Ok(Message::Close(_)) => break,
            Err(e) => {
                eprintln!("Error receiving message: {}", e);
                break;
            }
            _ => {}
        }
    }
} 