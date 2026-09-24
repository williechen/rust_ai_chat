pub mod http;
pub mod state;

use axum::{Router, routing::get};
use std::io::Result;

pub async fn application() -> Result<()> {
    let state = state::AppState::new();

    let app = Router::new()
        .route("/health", get(http::health))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000").await?;

    println!("Server running on http://127.0.0.1:3000");

    axum::serve(listener, app).await?;

    Ok(())
}
