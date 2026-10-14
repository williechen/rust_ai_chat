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
    pub schema_version: u16,
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
    /// 無效事件不占用去重鍵；Some(false) 代表本程序已看過。
    /// 僅用於記憶體契約學習，不是資料庫提交收據。
    pub fn accept_valid_event(&mut self, event: &EventInput) -> Option<bool> {
        if !valid_event(event) {
            return None;
        }
        Some(self.accept_once(EventIdentity::new(
            event.source_app.clone(),
            event.event_id.to_string(),
        )))
    }
}

// 事件類型長度以 UTF-8 位元組計算，避免儲存層限制不一致。
pub fn valid_event_type(value: &str) -> bool {
    !value.is_empty() && value.len() <= 128 && value.trim() == value
}

// 只做事件資料驗證；憑證、持久化與 ACK 各由真正 Adapter 負責。
pub fn valid_event(event: &EventInput) -> bool {
    event.schema_version == 1
        && !event.event_id.is_nil()
        && matches!(event.source_app.as_str(), "web_chat" | "desktop_pet")
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

async fn health() -> StatusCode {
    StatusCode::OK
}

pub fn build_app() -> Router {
    Router::new()
        .route("/healthz", get(health))
        .route("/api/v1/events", post(ingest))
        .layer(DefaultBodyLimit::max(4096))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn nil_event_id_is_not_accepted() {
        let invalid = EventInput {
            schema_version: 1,
            event_id: Uuid::nil(),
            source_app: "web_chat".into(),
            event_type: "chat.message.persisted".into(),
        };
        assert!(!valid_event(&invalid));
        let valid = EventInput {
            event_id: Uuid::new_v4(),
            ..invalid
        };
        assert!(valid_event(&valid));
    }

    #[test]
    fn rejects_bad_source_version_and_event_type() {
        let mut input = EventInput {
            schema_version: 1,
            event_id: Uuid::new_v4(),
            source_app: "web_chat".into(),
            event_type: "chat.message.persisted".into(),
        };
        assert!(valid_event(&input));
        input.source_app = "invalid_source".into();
        assert!(!valid_event(&input));
        input.source_app = "desktop_pet".into();
        input.schema_version = 2;
        assert!(!valid_event(&input));
        input.schema_version = 1;
        input.event_type = "   ".into();
        assert!(!valid_event(&input));
    }

    #[test]
    fn event_type_rejects_noncanonical_boundaries() {
        assert!(valid_event_type("chat.message.persisted"));
        assert!(!valid_event_type(" chat.message.persisted"));
        assert!(!valid_event_type("chat.message.persisted "));
        assert!(!valid_event_type("\n"));
        assert!(!valid_event_type(""));
        assert!(!valid_event_type(&"x".repeat(129)));
    }

    #[test]
    fn existing_event_validation_rejects_whitespace() {
        let event = EventInput {
            schema_version: 1,
            event_id: Uuid::new_v4(),
            source_app: "web_chat".into(),
            event_type: " chat.message.persisted".into(),
        };
        assert!(!valid_event(&event));
    }
    #[test]
    fn valid_event_is_accepted_once_per_source_and_id() {
        let id = Uuid::new_v4();
        let event = EventInput {
            schema_version: 1,
            event_id: id,
            source_app: "web_chat".into(),
            event_type: "chat.message.persisted".into(),
        };
        let mut dedup = InMemoryDeduplicator::default();
        assert_eq!(dedup.accept_valid_event(&event), Some(true));
        assert_eq!(dedup.accept_valid_event(&event), Some(false));
        let other_source = EventInput {
            source_app: "desktop_pet".into(),
            event_type: "pet.cleanup.succeeded".into(),
            ..event
        };
        assert_eq!(dedup.accept_valid_event(&other_source), Some(true));
    }

    #[test]
    fn invalid_event_does_not_reserve_dedup_key() {
        let id = Uuid::new_v4();
        let mut event = EventInput {
            schema_version: 2,
            event_id: id,
            source_app: "web_chat".into(),
            event_type: "chat.message.persisted".into(),
        };
        let mut dedup = InMemoryDeduplicator::default();
        assert_eq!(dedup.accept_valid_event(&event), None);
        event.schema_version = 1;
        assert_eq!(dedup.accept_valid_event(&event), Some(true));
    }
}
