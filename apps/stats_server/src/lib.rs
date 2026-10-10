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
pub struct EventInput {
    pub event_id: Uuid,
    pub source_app: String,
    pub event_type: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct EventIdentity {
    source_app: String,
    event_id: String,
}
impl EventIdentity {
    pub fn new(source_app: impl Into<String>, event_id: impl Into<String>) -> Self {
        Self {
            source_app: source_app.into(),
            event_id: event_id.into(),
        }
    }
}

#[derive(Default)]
pub struct InMemoryDeduplicator {
    seen: std::collections::HashSet<EventIdentity>,
}
impl InMemoryDeduplicator {
    pub fn accept_once(&mut self, identity: EventIdentity) -> bool {
        self.seen.insert(identity)
    }
}

// 事件類型長度以 UTF-8 位元組計算，避免儲存層限制不一致。
pub fn valid_event_type(value: &str) -> bool {
    let value = value.trim();
    !value.is_empty() && value.len() <= 128
}

// 純輸入檢查，不代表事件已完成持久化。
pub fn valid_event(event: &EventInput) -> bool {
    matches!(event.source_app.as_str(), "web_chat" | "desktop_pet")
        && valid_event_type(event.event_type.as_str())
}

// 有效事件尚未驗證身分／落 DB，不可假裝已處理。
pub async fn ingest(Json(event): Json<EventInput>) -> StatusCode {
    if !valid_event(&event) {
        return StatusCode::BAD_REQUEST;
    }
    let _ = event.event_id;
    StatusCode::NOT_IMPLEMENTED
}

pub async fn health() -> StatusCode {
    StatusCode::OK
}

pub fn build_app() -> Router {
    Router::new()
        .route("/healthz", get(health))
        .route("/api/v1/events", post(ingest))
        .layer(DefaultBodyLimit::max(4096))
}
