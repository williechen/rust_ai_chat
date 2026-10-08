use axum::{http::StatusCode, routing::{get, post}, Json, Router};
use serde::Deserialize;
use uuid::Uuid;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct EventInput {
    event_id: Uuid,
    source_app: String,
    event_type: String,
}

// 有效事件尚未驗證身分／落 DB，不可假裝已處理。
async fn ingest(Json(event): Json<EventInput>) -> StatusCode {
    if !matches!(event.source_app.as_str(), "web_chat" | "desktop_pet")
        || event.event_type.trim().is_empty()
    {
        return StatusCode::BAD_REQUEST;
    }
    let _ = event.event_id;
    StatusCode::NOT_IMPLEMENTED
}

async fn health() -> StatusCode { StatusCode::OK }

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let app = Router::new()
        .route("/healthz", get(health))
        .route("/api/v1/events", post(ingest));
    // 尚無 auth 和儲存，不可綁定公開地址。
    let listener = tokio::net::TcpListener::bind("127.0.0.1:3100").await?;
    axum::serve(listener, app).await?;
    Ok(())
}
