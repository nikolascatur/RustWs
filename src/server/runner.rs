use std::{collections::HashMap, sync::Arc};

use axum::{
    Extension, Router,
    extract::ws::{Message, WebSocket, WebSocketUpgrade},
    response::IntoResponse,
    routing::get,
};
use futures_util::{
    SinkExt,
    stream::{SplitSink, SplitStream, StreamExt},
};
use parking_lot::RwLock;
use tokio::{net::TcpListener, sync::mpsc};
use uuid::Uuid;

use crate::{
    actor::player::{ClientMessage, InputState, Player, Players, ServerMessage},
    server::service::game_loop,
};

pub async fn run_socket() {
    let players: Players = Arc::new(RwLock::new(HashMap::new()));
    let app = Router::new()
        .route("/ws", get(ws_handler))
        .layer(Extension(players.clone()));

    let tcp_listener = TcpListener::bind("0.0.0.0:3000").await.unwrap();
    tokio::spawn(game_loop(players));
    axum::serve(tcp_listener, app.into_make_service())
        .await
        .unwrap();
}

pub async fn ws_handler(
    ws: WebSocketUpgrade,
    Extension(players): Extension<Players>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_socket(socket, players))
}

async fn handle_socket(stream: WebSocket, players: Players) {
    let (mut sender, mut receiver) = stream.split();
    let (tx, mut rx) = mpsc::unbounded_channel::<Message>();

    // create user
    let id = Uuid::new_v4();
    let player = Arc::new(Player {
        id: id,
        tx: tx,
        x: 0.0,
        y: 0.0,
        angle: 0.0,
        vel: 0.0,
        hp: 100,
        latest_input: RwLock::new(None),
    });

    // user created share to others
    {
        let mut plays = players.write();
        plays.insert(id, player.clone());
    }

    //send message
    let welcome = ServerMessage::Welcome { id: id.to_string() };
    let _ = player
        .tx
        .send(Message::Text(serde_json::to_string(&welcome).unwrap()));

    // forward message dari receiver (rx) -> websocket sender
    let mut send_task = tokio::spawn(async move {
        while let Some(msg) = rx.recv().await {
            if sender.send(msg).await.is_err() {
                break;
            }
        }
    });

    // handle incoming message
    while let Some(Ok(msg)) = receiver.next().await {
        match msg {
            Message::Text(txt) => {
                if let Ok(cm) = serde_json::from_str::<ClientMessage>(&txt) {
                    match cm {
                        ClientMessage::Input {
                            seq,
                            throttle,
                            turn,
                            fire,
                        } => {
                            let mut li = player.latest_input.write();
                            *li = Some(InputState {
                                throttle: throttle,
                                turn: turn,
                                fire: fire,
                            })
                        }
                    }
                } else {
                }
            }
            Message::Binary(bin) => {}
            Message::Close(_) => break,
            _ => {}
        }
    }

    // cleanup create position broadcast delete
    {
        let mut pls = players.write();
        pls.remove(&id);
    }

    drop(player.tx.clone());
    let _ = send_task.await;

    // println!("Client Connected");
    // if stream
    //     .send(Message::Text("Welcome to Rust WebSocket server".into()))
    //     .await
    //     .is_err()
    // {
    //     println!("Faile to send welcome");
    //     return;
    // }

    // while let Some(Ok(msg)) = stream.recv().await {
    //     match msg {
    //         Message::Text(txt) => {
    //             println!("Receive text: {}", txt);
    //             let reply = format!("Server echo {}", txt);
    //             if stream.send(Message::Text(reply)).await.is_err() {
    //                 println!("Client disconect");
    //                 return;
    //             }
    //         }
    //         Message::Binary(bin) => {
    //             println!("📦 Received binary: {} bytes", bin.len());
    //         }
    //         Message::Close(_) => {
    //             println!("👋 Client disconnected");
    //             return;
    //         }
    //         _ => {}
    //     }
    // }
}
