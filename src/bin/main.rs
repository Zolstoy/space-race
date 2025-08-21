use std::{collections::HashMap, hash::Hash, sync::Arc};

use tokio::{net::{TcpListener, TcpStream}, sync::Mutex};
use tungstenite::protocol::Message;
use futures::{SinkExt, StreamExt};
use tokio_tungstenite::tungstenite;
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug)]
struct Player {
    position: (f32, f32, f32),
    velocity: (f32, f32, f32),
}

#[derive(Serialize, Deserialize, Debug)]
struct Others{
    position: (f32, f32, f32),
    velocity: (f32, f32, f32),
}

#[derive(Serialize, Deserialize, Debug)]
enum GameState {
    PlayerState(Player),
    OthersState(Others),
}

#[derive(Serialize, Deserialize, Debug)]
enum PlayerAction {
    Move((f32, f32, f32)),
    Stop,
}

#[derive(PartialEq, Eq, Hash, Debug)]
enum ClientState {
    Connected,
    Initialized,
}

struct Client{
    state_tx: crossbeam::channel::Sender<GameState>,
    action_rx: crossbeam::channel::Receiver<PlayerAction>,
    addr: std::net::SocketAddr,
    state: ClientState,
}

#[tokio::main]
async fn main() {
    let addr = "127.0.0.1:8080";
    let listener = TcpListener::bind(addr).await.expect(format!("Failed to bind to {}", addr).as_str());

    env_logger::init();

    log::info!("Server started on {}", addr);

    let clients = Arc::new(Mutex::new(HashMap::<u32, Client>::new()));

    let clients_clone = Arc::clone(&clients);
    tokio::spawn(async move {
        loop {
            {
                let mut clients_lock = clients_clone.lock().await;
                for (_id, client) in &*clients_lock {
                    if let Ok(action) = client.action_rx.try_recv() {
                        log::info!("Received action from {}: {:?}", client.addr, action);
                    }
                }

                for (id, client) in &mut *clients_lock {
                    if client.state == ClientState::Connected {
                        let state = GameState::PlayerState(Player {
                            position: (0.0, 0.0, 0.0),
                            velocity: (0.0, 0.0, 0.0),
                        });
                        if client.state_tx.send(state).is_err() {
                            log::warn!("Failed to send initial state to client {}", id);
                        }
                        client.state = ClientState::Initialized;
                    }
                }
            }


        }
    });

    while let Ok((stream, _)) = listener.accept().await {
        let (state_tx, state_rx) = crossbeam::channel::unbounded::<GameState>();
        let (action_tx, action_rx) = crossbeam::channel::unbounded::<PlayerAction>();
        let mut clients_lock = clients.lock().await;
        let client_id = clients_lock.len() as u32;
        clients_lock.insert(client_id, Client { state_tx, action_rx, addr: stream.peer_addr().expect("Invalid address"), state: ClientState::Connected });
        tokio::spawn(handle_connection(stream, state_rx, action_tx));
    }

    log::info!("Server ended gracefully");
}

async fn handle_connection(stream: TcpStream, state_rx: crossbeam::channel::Receiver<GameState>, action_tx: crossbeam::channel::Sender<PlayerAction>) {
    let addr = stream.peer_addr().expect("Invalid address");
    log::info!("New connection from {}", addr);

    let mut ws_stream = tokio_tungstenite::accept_async(stream)
        .await
        .expect("Failed to upgrade to WebSocket");

    // let player = Player {
    //     position: (0.0, 0.0, 0.0),
    //     velocity: (0.0, 0.0, 0.0),
    // };

    // let player_state_json = serde_json::to_string(&GameState::PlayerState(player))
    //     .expect("Failed to serialize player state");

    // ws_stream
    //     .send(Message::Text(player_state_json.into()))
    //     .await
    //     .expect("Failed to send message");

    tokio::select! {
        
    }
    while let Some(msg) = ws_stream.next().await {
        match msg {
            Ok(Message::Text(text)) => {
                log::info!("Received from {}: {}", addr, text);
                ws_stream
                    .send(Message::Text(format!("You said: {}", text).into()))
                    .await
                    .expect("Failed to send message");
            }
            Ok(Message::Close(_)) => {
                log::info!("Client {} disconnected", addr);
                break;
            }
            Err(e) => {
                log::info!("Error with {}: {}", addr, e);
                break;
            }
            _ => {}
        }
    }
}
