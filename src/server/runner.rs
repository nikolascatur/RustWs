use std::net::SocketAddr;

use axum::{
    Router,
    extract::ws::{Message, WebSocket, WebSocketUpgrade},
    response::{IntoResponse, Response},
    routing::get,
};
use tokio::net::TcpListener;

pub async fn run_socket() {
    let app = Router::new().route("/ws", get(ws_handler));
    let tcp_listener = TcpListener::bind("127.0.0.1:3000").await.unwrap();
    axum::serve(tcp_listener, app.into_make_service())
        .await
        .unwrap();
}

pub async fn ws_handler(ws: WebSocketUpgrade) -> impl IntoResponse {
    ws.on_upgrade(handle_socket)
}

async fn handle_socket(mut socket: WebSocket) {
    println!("Client Connected");
    if socket
        .send(Message::Text("Welcome to Rust WebSocket server".into()))
        .await
        .is_err()
    {
        println!("Faile to send welcome");
        return;
    }

    while let Some(Ok(msg)) = socket.recv().await {
        match msg {
            Message::Text(txt) => {
                println!("Receive text: {}", txt);
                let reply = format!("Server echo {}", txt);
                if socket.send(Message::Text(reply)).await.is_err() {
                    println!("Client disconect");
                    return;
                }
            }
            Message::Binary(bin) => {
                println!("📦 Received binary: {} bytes", bin.len());
            }
            Message::Close(_) => {
                println!("👋 Client disconnected");
                return;
            }
            _ => {}
        }
    }
}
