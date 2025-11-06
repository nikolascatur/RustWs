use axum::{Router, routing::get};

use crate::server::runner::{run_socket, ws_handler};

mod actor;
mod server;

#[tokio::main]
async fn main() {
    // println!("Hello, world!");
    run_socket().await;
}
