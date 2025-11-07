mod game;
mod state;
mod network;
mod storage;
mod ai;

use std::sync::{Arc, Mutex};

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let sessions = Arc::new(Mutex::new(state::Sessions::new()));
    let app = network::create_router(sessions);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080").await.unwrap();
    tracing::info!("Server running on http://0.0.0.0:8080");
    axum::serve(listener, app).await.unwrap();
}
