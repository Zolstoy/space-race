use tokio::net::{TcpListener, TcpStream};
use tungstenite::protocol::Message;
use futures::{SinkExt, StreamExt};
use tokio_tungstenite::tungstenite;

#[tokio::main]
async fn main() {
    let addr = "127.0.0.1:8080";
    let listener = TcpListener::bind(addr).await.expect(format!("Failed to bind to {}", addr).as_str());

    println!("Server started on {}", addr);

    while let Ok((stream, _)) = listener.accept().await {
        tokio::spawn(handle_connection(stream));
    }

    println!("Server ended gracefully");
}

async fn handle_connection(stream: TcpStream) {
    let addr = stream.peer_addr().expect("Invalid address");
    println!("New connection from {}", addr);

    let mut ws_stream = tokio_tungstenite::accept_async(stream)
        .await
        .expect("Failed to upgrade to WebSocket");

    // Send a welcome message
    ws_stream
        .send(Message::Text("Hello!".into()))
        .await
        .expect("Failed to send message");

    // Listen for messages from the client
    while let Some(msg) = ws_stream.next().await {
        match msg {
            Ok(Message::Text(text)) => {
                println!("Received from {}: {}", addr, text);
                // RRespond with an echo
                ws_stream
                    .send(Message::Text(format!("You said: {}", text).into()))
                    .await
                    .expect("Failed to send message");
            }
            Ok(Message::Close(_)) => {
                println!("Client {} disconnected", addr);
                break;
            }
            Err(e) => {
                println!("Error with {}: {}", addr, e);
                break;
            }
            _ => {}
        }
    }
}
