#![forbid(unsafe_code)]

use std::{collections::HashMap, hash::Hash, sync::Arc};

use futures::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use tokio::{
    net::{TcpListener, TcpStream},
    sync::{mpsc, Mutex},
};
use tokio_stream::wrappers::ReceiverStream;
use tokio_tungstenite::tungstenite;
use tungstenite::protocol::Message;

#[derive(Serialize, Deserialize, Debug)]
struct Player {
    position: (f32, f32, f32),
    velocity: (f32, f32, f32),
}

#[derive(Serialize, Deserialize, Debug)]
struct Others {
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

struct Client {
    state_tx: mpsc::Sender<GameState>,
    action_rx: crossbeam::channel::Receiver<PlayerAction>,
    addr: std::net::SocketAddr,
    state: ClientState,
}

async fn handle_connection(
    stream: TcpStream,
    state_rx: mpsc::Receiver<GameState>,
    _action_tx: crossbeam::channel::Sender<PlayerAction>,
) -> () {
    let addr = stream.peer_addr().expect("Invalid address");
    log::info!("New connection from {}", addr);

    let mut ws_stream = tokio_tungstenite::accept_async(stream)
        .await
        .expect("Failed to upgrade to WebSocket");

    log::info!("WebSocket connection established: {}", addr);
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

    let mut state_stream = ReceiverStream::new(state_rx);
    loop {
        tokio::select! {
            state = state_stream.next() => {
                let state_json = serde_json::to_string(&state)
                    .expect("Failed to serialize game state");
                ws_stream
                    .send(Message::Text(state_json.into()))
                    .await
                    .expect("Failed to send message");
            }
            Some(msg) = ws_stream.next() => {
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
                        return;
                    }
                    Err(e) => {
                        log::info!("Error with {}: {}", addr, e);
                        return;
                    }
                    _ => {}
                }
            }
        }
    }
}

pub async fn run_with_listener(listener: TcpListener) {
    let addr = listener.local_addr().unwrap();
    let port = addr.port();

    log::info!("Server started on {}:{}", addr.ip(), port);

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
                        if client.state_tx.send(state).await.is_err() {
                            log::warn!("Failed to send initial state to client {}", id);
                        }
                        client.state = ClientState::Initialized;
                    }
                }
            }
        }
    });

    while let Ok((stream, _)) = listener.accept().await {
        let (state_tx, state_rx) = mpsc::channel::<GameState>(100);
        let (action_tx, action_rx) = crossbeam::channel::bounded::<PlayerAction>(100);
        let mut clients_lock = clients.lock().await;
        let client_id = clients_lock.len() as u32;
        clients_lock.insert(
            client_id,
            Client {
                state_tx,
                action_rx,
                addr: stream.peer_addr().expect("Invalid address"),
                state: ClientState::Connected,
            },
        );
        tokio::spawn(handle_connection(stream, state_rx, action_tx));
    }

    log::info!("Server ended gracefully");
}

pub async fn run(addr: &str, port: u16) {
    let listener = TcpListener::bind((addr, port))
        .await
        .expect(format!("Failed to bind to {}:{}", addr, port).as_str());

    run_with_listener(listener).await;
}
