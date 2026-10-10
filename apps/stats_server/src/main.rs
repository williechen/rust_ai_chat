use axum::{
    Json, Router,
    extract::DefaultBodyLimit,
    http::StatusCode,
    routing::{get, post},
};
use serde::Deserialize;
use uuid::Uuid;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct EventInput {
    event_id: Uuid,
    source_app: String,
    event_type: String,
}

// 事件類型長度以 UTF-8 位元組計算，避免儲存層限制不一致。
fn valid_event_type(value: &str) -> bool {
    let value = value.trim();
    !value.is_empty() && value.len() <= 128
}

// 純輸入檢查，不代表事件已完成持久化。
fn valid_event(event: &EventInput) -> bool {
    matches!(event.source_app.as_str(), "web_chat" | "desktop_pet")
        && valid_event_type(event.event_type.as_str())
}

// 有效事件尚未驗證身分／落 DB，不可假裝已處理。
async fn ingest(Json(event): Json<EventInput>) -> StatusCode {
    if !valid_event(&event) {
        return StatusCode::BAD_REQUEST;
    }
    let _ = event.event_id;
    StatusCode::NOT_IMPLEMENTED
}

async fn health() -> StatusCode {
    StatusCode::OK
}

fn build_app() -> Router {
    Router::new()
        .route("/healthz", get(health))
        .route("/api/v1/events", post(ingest))
        .layer(DefaultBodyLimit::max(4096))
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let app = build_app();
    // 尚無 auth 和儲存，不可綁定公開地址。
    let listener = tokio::net::TcpListener::bind("127.0.0.1:3100").await?;
    axum::serve(listener, app).await?;
    Ok(())
}
