use axum::{
    Json, Router,
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

// 純輸入檢查，不代表事件已完成持久化。
fn valid_event(event: &EventInput) -> bool {
    matches!(event.source_app.as_str(), "web_chat" | "desktop_pet")
        && !event.event_type.trim().is_empty()
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

#[cfg(test)]
mod tests {
    use super::*;
    fn input(source: &str, kind: &str) -> EventInput {
        EventInput {
            event_id: Uuid::new_v4(),
            source_app: source.into(),
            event_type: kind.into(),
        }
    }
    #[test]
    fn validates_event_inputs() {
        assert!(valid_event(&input("web_chat", "message_sent")));
        assert!(valid_event(&input("desktop_pet", "scan_preview")));
        assert!(!valid_event(&input("unknown", "message_sent")));
        assert!(!valid_event(&input("web_chat", "  ")));
    }
    #[tokio::test]
    async fn valid_event_remains_unimplemented() {
        assert_eq!(
            ingest(Json(input("web_chat", "message_sent"))).await,
            StatusCode::NOT_IMPLEMENTED
        );
    }
    #[tokio::test]
    async fn invalid_event_is_rejected() {
        assert_eq!(
            ingest(Json(input("unknown", "x"))).await,
            StatusCode::BAD_REQUEST
        );
    }
}
