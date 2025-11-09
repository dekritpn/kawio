use kawio::*;

use std::env;
use std::sync::{Arc, Mutex};
use tower_http::services::ServeDir;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();

    let port = env::var("PORT").unwrap_or_else(|_| "8080".to_string());
    let address = format!("0.0.0.0:{}", port);

    let sessions = Arc::new(Mutex::new(state::Sessions::new()));
    let api_router = network::create_router(sessions);
    let app = api_router.fallback_service(ServeDir::new("web"));

    let listener = tokio::net::TcpListener::bind(&address).await?;
    tracing::info!("Server running on http://{}", address);
    axum::serve(listener, app).await?;
    Ok(())
}
